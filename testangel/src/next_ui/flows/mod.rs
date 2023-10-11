use std::{fs, path::PathBuf, rc::Rc, sync::Arc};

use adw::prelude::*;
use relm4::{
    adw, factory::FactoryVecDeque, gtk, Component, ComponentController, ComponentParts,
    ComponentSender, Controller, RelmWidgetExt, SimpleComponent, prelude::DynamicIndex,
};
use rust_i18n::t;
use testangel::{
    action_loader::ActionMap,
    ipc::EngineList,
    types::{AutomationFlow, VersionedFile, ActionParameterSource, ActionConfiguration},
};

mod action_component;
pub mod header;

pub enum SaveOrOpenFlowError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    FlowNotVersionCompatible,
    MissingAction(usize, String),
}

impl ToString for SaveOrOpenFlowError {
    fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => t!("flows.save-or-open-flow-error.io-error", error = e),
            Self::ParsingError(e) => t!("flows.save-or-open-flow-error.parsing-error", error = e),
            Self::SerializingError(e) => {
                t!("flows.save-or-open-flow-error.serializing-error", error = e)
            }
            Self::FlowNotVersionCompatible => {
                t!("flows.save-or-open-flow-error.flow-not-version-compatible")
            }
            Self::MissingAction(step, e) => t!(
                "flows.save-or-open-flow-error.missing-action",
                step = step + 1,
                error = e
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FlowInputs {
    /// Do nothing
    NoOp,
    /// The map of actions has changed and should be updated
    ActionsMapChanged(Arc<ActionMap>),
    /// Create a new flow
    NewFlow,
    /// Actually create the new flow
    _NewFlow,
    /// Prompt the user to open a flow. This will ask to save first if needed.
    OpenFlow,
    /// Actually show the user the open file dialog
    _OpenFlow,
    /// Actually open a flow after the user has finished selecting
    __OpenFlow,
    /// Save the flow, prompting if needed to set file path
    SaveFlow,
    /// Save the flow as a new file, always prompting for a file path
    SaveAsFlow,
    /// Ask where to save if needed, then save
    _SaveFlowThen(Box<FlowInputs>),
    /// Actually write the flow to disk, then emit then input
    __SaveFlowThen(Box<FlowInputs>),
    /// Close the flow, prompting if needing to save first
    CloseFlow,
    /// Actually close the flow
    _CloseFlow,
    /// Add the step with the ID provided
    AddStep(String),
    /// Update the UI steps from the open flow. This will clear first and overwrite any changes!
    UpdateStepsFromModel,
    /// Remove the step with the provided index, resetting all references to it.
    RemoveStep(DynamicIndex),
    /// Remove the step with the provided index, but change references to it to a temporary value (`usize::MAX`)
    /// that can be set again with [`FlowInputs::PasteStep`].
    /// This doesn't refresh the UI until Paste is called.
    CutStep(DynamicIndex),
    /// Insert a step at the specified index and set references back to the correct step.
    /// This refreshes the UI.
    PasteStep(usize, ActionConfiguration),
}

#[derive(Debug)]
pub struct FlowsModel {
    action_map: Arc<ActionMap>,
    engine_list: Arc<EngineList>,

    open_flow: Option<AutomationFlow>,
    open_path: Option<PathBuf>,
    needs_saving: bool,
    header: Rc<Controller<header::FlowsHeader>>,
    live_actions_list: FactoryVecDeque<action_component::ActionComponent>,

    open_dialog: gtk::FileChooserDialog,
    save_dialog: gtk::FileChooserDialog,
}

impl FlowsModel {
    /// Get an [`Rc`] clone of the header controller
    pub fn header_controller_rc(&self) -> Rc<Controller<header::FlowsHeader>> {
        self.header.clone()
    }

    /// Create the absolute barebones of a message dialog, allowing for custom button and response mapping.
    fn create_message_dialog_skeleton<S>(&self, title: S, message: S) -> adw::MessageDialog
    where
        S: AsRef<str>,
    {
        adw::MessageDialog::builder()
            .transient_for(&self.header.widget().toplevel_window().expect(
                "FlowsModel::create_message_dialog cannot be called until the header is attached",
            ))
            .title(title.as_ref())
            .heading(title.as_ref())
            .body(message.as_ref())
            .modal(true)
            .build()
    }

    /// Create a message dialog attached to the toplevel window. This includes default implementations of an 'OK' button.
    fn create_message_dialog<S>(&self, title: S, message: S) -> adw::MessageDialog
    where
        S: AsRef<str>,
    {
        let dialog = self.create_message_dialog_skeleton(title, message);
        dialog.add_response("ok", &t!("ok"));
        dialog.set_default_response(Some("ok"));
        dialog.set_close_response("ok");
        dialog
    }

    /// Just open a brand new flow
    fn new_flow(&mut self) {
        self.open_path = None;
        self.needs_saving = true;
        self.open_flow = Some(AutomationFlow::default());
    }

    /// Open a flow. This does not ask to save first.
    fn open_flow(&mut self, file: PathBuf) -> Result<Vec<usize>, SaveOrOpenFlowError> {
        let data = &fs::read_to_string(&file).map_err(SaveOrOpenFlowError::IoError)?;

        let versioned_file: VersionedFile =
            ron::from_str(data).map_err(SaveOrOpenFlowError::ParsingError)?;
        if versioned_file.version() != 1 {
            return Err(SaveOrOpenFlowError::FlowNotVersionCompatible);
        }

        let mut flow: AutomationFlow =
            ron::from_str(data).map_err(SaveOrOpenFlowError::ParsingError)?;
        if flow.version() != 1 {
            return Err(SaveOrOpenFlowError::FlowNotVersionCompatible);
        }
        let mut steps_reset = vec![];
        for (step, ac) in flow.actions.iter_mut().enumerate() {
            match self.action_map.get_action_by_id(&ac.action_id) {
                None => {
                    return Err(SaveOrOpenFlowError::MissingAction(
                        step,
                        ac.action_id.clone(),
                    ))
                }
                Some(action) => {
                    // Check that action parameters haven't changed. If they have, reset values.
                    if ac.update(action) {
                        steps_reset.push(step + 1);
                    }
                }
            }
        }
        self.open_flow = Some(flow);
        self.open_path = Some(file);
        self.needs_saving = false;
        log::debug!("New flow open.");
        log::debug!("Flow: {:?}", self.open_flow);
        Ok(steps_reset)
    }

    /// Ask the user if they want to save this file. If they response yes, this will also trigger the save function.
    /// This function will only ask the user if needed, otherwise it will emit immediately.
    fn prompt_to_save(&self, sender: &relm4::Sender<FlowInputs>, then: FlowInputs) {
        if self.needs_saving {
            let question = self.create_message_dialog_skeleton(
                t!("flows.save-before"),
                t!("flows.save-before-message"),
            );
            question.add_response("discard", &t!("discard"));
            question.add_response("save", &t!("save"));
            question.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
            question.set_default_response(Some("save"));
            question.set_close_response("discard");
            let sender = sender.clone();
            question.connect_response(Some("save"), move |_, _| {
                sender.emit(FlowInputs::_SaveFlowThen(Box::new(then.clone())));
            });
            question.show();
        } else {
            sender.emit(then);
        }
    }

    /// Ask the user where to save the flow, or just save if that's good enough
    fn ask_where_to_save(
        &mut self,
        sender: &relm4::Sender<FlowInputs>,
        always_ask_where: bool,
        then: FlowInputs,
    ) {
        if always_ask_where || self.open_path.is_none() {
            // Ask where
            self.save_dialog.show();
        } else {
            sender.emit(FlowInputs::_SaveFlowThen(Box::new(then)));
        }
    }

    /// Just save the flow to disk with the current `open_path` as the destination
    fn save_flow(&mut self) -> Result<(), SaveOrOpenFlowError> {
        let save_path = self.open_path.as_ref().unwrap();
        let data = ron::to_string(self.open_flow.as_ref().unwrap())
            .map_err(SaveOrOpenFlowError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenFlowError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Close this flow without checking first
    fn close_flow(&mut self) {
        self.open_flow = None;
        self.open_path = None;
        self.needs_saving = false;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsModel {
    type Init = (
        gtk::FileChooserDialog, // A file chooser transient for the parent window
        gtk::FileChooserDialog, // A file chooser transient for the parent window
        Arc<ActionMap>,
        Arc<EngineList>,
    );
    type Input = FlowInputs;
    type Output = ();

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,

                gtk::Label {
                    set_label: "Drag-and-drop is not yet implemented to reorder steps.",
                },

                adw::StatusPage {
                    set_title: &t!("flows.nothing-open"),
                    set_description: Some(&t!("flows.nothing-open-description")),
                    set_icon_name: Some(relm4_icons::icon_name::LIGHTBULB),
                    #[watch]
                    set_visible: model.open_flow.is_none(),
                    set_vexpand: true,
                },

                #[local_ref]
                live_actions_list -> gtk::ListBox {

                },
            },
        },

        #[local_ref]
        open_dialog -> gtk::FileChooserDialog {
            set_title: Some(&t!("flows.open")),
            set_action: gtk::FileChooserAction::Open,
            set_modal: true,
            add_button[gtk::ResponseType::Ok]: &t!("open"),

            connect_response[sender] => move |_, response| {
                if response == gtk::ResponseType::Ok {
                    sender.input(FlowInputs::__OpenFlow);
                }
            },
        },

        #[local_ref]
        save_dialog -> gtk::FileChooserDialog {
            set_title: Some(&t!("flows.save")),
            set_action: gtk::FileChooserAction::Save,
            set_modal: true,
            add_button[gtk::ResponseType::Ok]: &t!("save"),

            connect_response[sender] => move |_, response| {
                if response == gtk::ResponseType::Ok {
                    sender.input(FlowInputs::_SaveFlowThen(Box::new(FlowInputs::NoOp)));
                }
            },
        },
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Rc::new(header::FlowsHeader::builder().launch(init.2.clone()).forward(
            sender.input_sender(),
            |msg| match msg {
                header::FlowsHeaderOutput::NewFlow => FlowInputs::NewFlow,
                header::FlowsHeaderOutput::OpenFlow => FlowInputs::OpenFlow,
                header::FlowsHeaderOutput::SaveFlow => FlowInputs::SaveFlow,
                header::FlowsHeaderOutput::SaveAsFlow => FlowInputs::SaveAsFlow,
                header::FlowsHeaderOutput::CloseFlow => FlowInputs::CloseFlow,
                header::FlowsHeaderOutput::RunFlow => FlowInputs::NoOp,
                header::FlowsHeaderOutput::AddStep(step) => FlowInputs::AddStep(step),
            },
        ));

        let model = FlowsModel {
            action_map: init.2,
            engine_list: init.3,
            open_flow: None,
            open_path: None,
            needs_saving: false,
            open_dialog: init.0,
            save_dialog: init.1,
            header,
            live_actions_list: FactoryVecDeque::new(
                gtk::ListBox::builder().css_classes(["boxed-list"]).build(),
                sender.input_sender(),
            ),
        };

        // Trigger update actions from model
        sender.input(FlowInputs::UpdateStepsFromModel);

        let live_actions_list = model.live_actions_list.widget();
        let open_dialog = &model.open_dialog;
        let save_dialog = &model.save_dialog;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowInputs::NoOp => (),
            FlowInputs::ActionsMapChanged(new_map) => {
                self.action_map = new_map.clone();
                self.header.emit(header::FlowsHeaderInput::ActionsMapChanged(new_map));
            }
            FlowInputs::NewFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_NewFlow);
            }
            FlowInputs::_NewFlow => {
                self.new_flow();
            }
            FlowInputs::OpenFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_OpenFlow);
            }
            FlowInputs::_OpenFlow => {
                self.open_dialog.show();
            }
            FlowInputs::__OpenFlow => {
                self.open_dialog.hide();

                if let Some(file) = self.open_dialog.file() {
                    if let Some(path) = file.path() {
                        match self.open_flow(path) {
                            Ok(changes) => {
                                // Reload UI
                                sender.input(FlowInputs::UpdateStepsFromModel);

                                if !changes.is_empty() {
                                    let changed_steps = changes
                                        .iter()
                                        .map(|step| step.to_string())
                                        .collect::<Vec<_>>()
                                        .join(",");
                                    self.create_message_dialog(
                                        t!("flows.action-changed"),
                                        t!("flows.action-changed-message", steps = changed_steps),
                                    )
                                    .show();
                                }
                            }
                            Err(e) => {
                                // Show error dialog
                                self.create_message_dialog(
                                    t!("flows.error-opening"),
                                    e.to_string(),
                                )
                                .show();
                            }
                        }
                    }
                }
            }
            FlowInputs::SaveFlow => {
                if self.open_flow.is_some() {
                    self.ask_where_to_save(sender.input_sender(), false, FlowInputs::NoOp);
                }
            }
            FlowInputs::SaveAsFlow => {
                if self.open_flow.is_some() {
                    self.ask_where_to_save(sender.input_sender(), true, FlowInputs::NoOp);
                }
            }
            FlowInputs::_SaveFlowThen(then) => {
                self.ask_where_to_save(sender.input_sender(), false, *then);
            }
            FlowInputs::__SaveFlowThen(then) => {
                if let Err(e) = self.save_flow() {
                    self.create_message_dialog(t!("flows.error-saving"), e.to_string())
                        .show();
                } else {
                    sender.input_sender().emit(*then);
                }
            }
            FlowInputs::CloseFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_CloseFlow);
            }
            FlowInputs::_CloseFlow => {
                self.close_flow();
            }

            FlowInputs::AddStep(step_id) => {
                if self.open_flow.is_none() {
                    self.new_flow();
                }

                // unwrap rationale: we've just guaranteed a flow is open
                let flow = self.open_flow.as_mut().unwrap();
                // unwrap rationale: the header can't ask to add an action that doesn't exist
                flow.actions.push(ActionConfiguration::from(self.action_map.get_action_by_id(&step_id).unwrap()));
                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }

            FlowInputs::UpdateStepsFromModel => {
                let mut live_list = self.live_actions_list.guard();
                live_list.clear();
                if let Some(flow) = &self.open_flow {
                    for (step, config) in flow.actions.iter().enumerate() {
                        live_list.push_back(action_component::ActionComponentInitialiser {
                            step,
                            config: config.clone(),
                            action: self.action_map.get_action_by_id(&config.action_id).unwrap(), // rationale: we have already checked the actions are here when the file is opened
                        });
                    }
                }
            }

            FlowInputs::RemoveStep(step_idx) => {
                let idx = step_idx.current_index();
                let flow = self.open_flow.as_mut().unwrap();
                log::info!("Deleting step {}", idx + 1);

                flow.actions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in flow.actions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => *source = ActionParameterSource::Literal,
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }
            FlowInputs::CutStep(step_idx) => {
                let idx = step_idx.current_index();
                let flow = self.open_flow.as_mut().unwrap();
                log::info!("Deleting step {}", idx + 1);

                flow.actions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in flow.actions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => *from_step = usize::MAX,
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }
            }
            FlowInputs::PasteStep(idx, config) => {
                let flow = self.open_flow.as_mut().unwrap();
                let idx = idx.max(0).min(flow.actions.len());
                flow.actions.insert(idx, config);

                // Remove references to step and renumber references above step to one less than they were
                for step in flow.actions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            if *from_step == usize::MAX {
                                *from_step = idx;
                            } else if *from_step >= idx {
                                *from_step += 1;
                            }
                        }
                    }
                }

                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }
        }
    }
}
