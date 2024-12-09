use std::{collections::HashMap, fmt, fs, path::PathBuf, rc::Rc, sync::Arc};

use adw::prelude::*;
use relm4::{
    adw, component::Connector, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
};
use testangel::{
    action_loader::ActionMap,
    ipc::EngineList,
    types::{ActionConfiguration, ActionParameterSource, AutomationFlow, VersionedFile},
};

use crate::ui::flows::action_component::ActionComponentOutput;

use super::{file_filters, lang};

mod action_component;
mod execution_dialog;
pub mod header;

pub enum SaveOrOpenFlowError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    FlowNotVersionCompatible,
    MissingAction(usize, String),
}

impl fmt::Display for SaveOrOpenFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => lang::lookup_with_args("flow-save-open-error-io-error", {
                    let mut map = HashMap::new();
                    map.insert("error", e.to_string().into());
                    map
                }),
                Self::ParsingError(e) => {
                    lang::lookup_with_args("flow-save-open-error-parsing-error", {
                        let mut map = HashMap::new();
                        map.insert("error", e.to_string().into());
                        map
                    })
                }
                Self::SerializingError(e) => {
                    lang::lookup_with_args("flow-save-open-error-serializing-error", {
                        let mut map = HashMap::new();
                        map.insert("error", e.to_string().into());
                        map
                    })
                }
                Self::FlowNotVersionCompatible => {
                    lang::lookup("flow-save-open-error-flow-not-version-compatible")
                }
                Self::MissingAction(step, e) => {
                    lang::lookup_with_args("flow-save-open-error-missing-action", {
                        let mut map = HashMap::new();
                        map.insert("step", (step + 1).into());
                        map.insert("error", e.to_string().into());
                        map
                    })
                }
            }
        )
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
    __OpenFlow(PathBuf),
    /// Save the flow, prompting if needed to set file path
    SaveFlow,
    /// Save the flow as a new file, always prompting for a file path
    SaveAsFlow,
    /// Ask where to save if needed, then save
    _SaveFlowThen(Box<FlowInputs>),
    /// Actually write the flow to disk, then emit then input
    __SaveFlowThen(PathBuf, Box<FlowInputs>),
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
    /// Move a step from the index to a position offset (param 3) from a new index (param 2).
    MoveStep(DynamicIndex, DynamicIndex, isize),
    /// Start the flow exection
    RunFlow,
    /// The [`ActionConfiguration`] has changed for the step indicated by the [`DynamicIndex`].
    /// This does not refresh the UI.
    ConfigUpdate(DynamicIndex, ActionConfiguration),
}

#[derive(Debug)]
pub enum FlowOutputs {}

#[derive(Debug)]
pub struct FlowsModel {
    action_map: Arc<ActionMap>,
    engine_list: Arc<EngineList>,

    open_flow: Option<AutomationFlow>,
    open_path: Option<PathBuf>,
    needs_saving: bool,
    header: Rc<Controller<header::FlowsHeader>>,
    live_actions_list: FactoryVecDeque<action_component::ActionComponent>,

    execution_dialog: Option<Connector<execution_dialog::ExecutionDialog>>,
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
        dialog.add_response("ok", &lang::lookup("ok"));
        dialog.set_default_response(Some("ok"));
        dialog.set_close_response("ok");
        dialog
    }

    /// Just open a brand new flow
    fn new_flow(&mut self) {
        self.open_path = None;
        self.needs_saving = true;
        self.open_flow = Some(AutomationFlow::default());
        self.header.emit(header::FlowsHeaderInput::ChangeFlowOpen(
            self.open_flow.is_some(),
        ));
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
        self.header.emit(header::FlowsHeaderInput::ChangeFlowOpen(
            self.open_flow.is_some(),
        ));
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
                lang::lookup("flow-save-before"),
                lang::lookup("flow-save-before-message"),
            );
            question.add_response("discard", &lang::lookup("discard"));
            question.add_response("save", &lang::lookup("save"));
            question.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
            question.set_default_response(Some("save"));
            question.set_close_response("discard");
            let sender_c = sender.clone();
            let then_c = then.clone();
            question.connect_response(Some("save"), move |_, _| {
                sender_c.emit(FlowInputs::_SaveFlowThen(Box::new(then_c.clone())));
            });
            let sender_c = sender.clone();
            question.connect_response(Some("discard"), move |_, _| {
                sender_c.emit(then.clone());
            });
            question.set_visible(true);
        } else {
            sender.emit(then);
        }
    }

    /// Ask the user where to save the flow, or just save if that's good enough
    fn ask_where_to_save(
        &mut self,
        sender: &relm4::Sender<FlowInputs>,
        transient_for: &impl IsA<gtk::Window>,
        always_ask_where: bool,
        then: FlowInputs,
    ) {
        if always_ask_where || self.open_path.is_none() {
            // Ask where
            let dialog = gtk::FileDialog::builder()
                .modal(true)
                .title(lang::lookup("header-save"))
                .initial_folder(&gtk::gio::File::for_path(
                    std::env::var("TA_FLOW_DIR").unwrap_or(".".to_string()),
                ))
                .filters(&file_filters::filter_list(vec![
                    file_filters::flows(),
                    file_filters::all(),
                ]))
                .build();

            let sender_c = sender.clone();
            dialog.save(
                Some(transient_for),
                Some(&relm4::gtk::gio::Cancellable::new()),
                move |res| {
                    if let Ok(file) = res {
                        let mut path = file.path().unwrap();
                        path.set_extension("taflow");
                        sender_c.emit(FlowInputs::__SaveFlowThen(path, Box::new(then.clone())));
                    }
                },
            );
        } else {
            sender.emit(FlowInputs::__SaveFlowThen(
                self.open_path.clone().unwrap(),
                Box::new(then),
            ));
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
        self.live_actions_list.guard().clear();
        self.header.emit(header::FlowsHeaderInput::ChangeFlowOpen(
            self.open_flow.is_some(),
        ));
    }
}

#[relm4::component(pub)]
impl Component for FlowsModel {
    type Init = (Arc<ActionMap>, Arc<EngineList>);
    type Input = FlowInputs;
    type Output = FlowOutputs;
    type CommandOutput = ();

    view! {
        #[root]
        toast_target = adw::ToastOverlay {
            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,

                    adw::StatusPage {
                        set_title: &lang::lookup("nothing-open"),
                        set_description: Some(&lang::lookup("flow-nothing-open-description")),
                        set_icon_name: Some(relm4_icons::icon_names::LIGHTBULB),
                        #[watch]
                        set_visible: model.open_flow.is_none(),
                        set_vexpand: true,
                    },

                    #[local_ref]
                    live_actions_list -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                    },
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Rc::new(
            header::FlowsHeader::builder()
                .launch(init.0.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    header::FlowsHeaderOutput::NewFlow => FlowInputs::NewFlow,
                    header::FlowsHeaderOutput::OpenFlow => FlowInputs::OpenFlow,
                    header::FlowsHeaderOutput::SaveFlow => FlowInputs::SaveFlow,
                    header::FlowsHeaderOutput::SaveAsFlow => FlowInputs::SaveAsFlow,
                    header::FlowsHeaderOutput::CloseFlow => FlowInputs::CloseFlow,
                    header::FlowsHeaderOutput::RunFlow => FlowInputs::RunFlow,
                    header::FlowsHeaderOutput::AddStep(step) => FlowInputs::AddStep(step),
                }),
        );

        let model = FlowsModel {
            action_map: init.0,
            engine_list: init.1,
            open_flow: None,
            open_path: None,
            needs_saving: false,
            execution_dialog: None,
            header,
            live_actions_list: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), |output| match output {
                    ActionComponentOutput::Remove(idx) => FlowInputs::RemoveStep(idx),
                    ActionComponentOutput::Cut(idx) => FlowInputs::CutStep(idx),
                    ActionComponentOutput::Paste(idx, step) => FlowInputs::PasteStep(idx, step),
                    ActionComponentOutput::ConfigUpdate(step, config) => {
                        FlowInputs::ConfigUpdate(step, config)
                    }
                    ActionComponentOutput::MoveStep(from, to, offset) => {
                        FlowInputs::MoveStep(from, to, offset)
                    }
                }),
        };

        // Trigger update actions from model
        sender.input(FlowInputs::UpdateStepsFromModel);

        let live_actions_list = model.live_actions_list.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            FlowInputs::NoOp => (),
            FlowInputs::ActionsMapChanged(new_map) => {
                self.action_map = new_map.clone();
                self.header
                    .emit(header::FlowsHeaderInput::ActionsMapChanged(new_map));

                // This may have changed action parameters, so check again.
                let mut close_flow = false;
                let mut steps_reset = vec![];
                if let Some(flow) = &mut self.open_flow {
                    let actions_clone = flow.actions.clone();
                    for (step, ac) in flow.actions.iter_mut().enumerate() {
                        match self.action_map.get_action_by_id(&ac.action_id) {
                            None => {
                                close_flow = true;
                            }
                            Some(action) => {
                                // Check that action parameters haven't changed. If they have, reset values.
                                if ac.update(action.clone()) {
                                    steps_reset.push(step);
                                }

                                // Check that the references from this AC to another don't now violate types
                                for (p_id, src) in &mut ac.parameter_sources {
                                    if let ActionParameterSource::FromOutput(other_step, output) =
                                        src
                                    {
                                        let (_name, kind) = &action.parameters()[*p_id];
                                        // Check that parameter from step->output is of type kind
                                        if let Some(other_ac) = actions_clone.get(*other_step) {
                                            if let Some(other_action) = &self
                                                .action_map
                                                .get_action_by_id(&other_ac.action_id)
                                            {
                                                if let Some((_name, other_output_kind)) =
                                                    other_action.outputs().get(*output)
                                                {
                                                    if kind != other_output_kind {
                                                        // Reset to literal
                                                        steps_reset.push(step);
                                                        *src = ActionParameterSource::Literal;
                                                    }
                                                }
                                            }
                                        }
                                        // If any of these if's fail, then the main loop will catch and fail later.
                                    }
                                }
                            }
                        }
                    }
                    sender.input(FlowInputs::UpdateStepsFromModel);
                }
                if !steps_reset.is_empty() {
                    let toast =
                        adw::Toast::new(&lang::lookup_with_args("flow-action-changed-message", {
                            let mut map = HashMap::new();
                            map.insert("stepCount", steps_reset.len().into());
                            map.insert(
                                "steps",
                                steps_reset
                                    .iter()
                                    .map(|i| (i + 1).to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                                    .into(),
                            );
                            map
                        }));
                    toast.set_timeout(0); // indefinte so it can be seen when switching back
                    widgets.toast_target.add_toast(toast);
                }
                if close_flow {
                    self.close_flow();
                }
            }
            FlowInputs::ConfigUpdate(step, new_config) => {
                // unwrap rationale: config updates can't happen if nothing is open
                let flow = self.open_flow.as_mut().unwrap();
                flow.actions[step.current_index()] = new_config;
                self.needs_saving = true;
            }
            FlowInputs::NewFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_NewFlow);
            }
            FlowInputs::_NewFlow => {
                self.new_flow();
                sender.input(FlowInputs::UpdateStepsFromModel);
            }
            FlowInputs::OpenFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_OpenFlow);
            }
            FlowInputs::_OpenFlow => {
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-open"))
                    .filters(&file_filters::filter_list(vec![
                        file_filters::flows(),
                        file_filters::all(),
                    ]))
                    .initial_folder(&gtk::gio::File::for_path(
                        std::env::var("TA_FLOW_DIR").unwrap_or(".".to_string()),
                    ))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            sender_c.input(FlowInputs::__OpenFlow(path));
                        }
                    },
                );
            }
            FlowInputs::__OpenFlow(path) => {
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
                                lang::lookup("flow-action-changed"),
                                lang::lookup_with_args("flow-action-changed-message", {
                                    let mut map = HashMap::new();
                                    map.insert("stepCount", changes.len().into());
                                    map.insert("steps", changed_steps.into());
                                    map
                                }),
                            )
                            .set_visible(true);
                        }
                    }
                    Err(e) => {
                        // Show error dialog
                        self.create_message_dialog(
                            lang::lookup("flow-error-opening"),
                            e.to_string(),
                        )
                        .set_visible(true);
                    }
                }
            }
            FlowInputs::SaveFlow => {
                if self.open_flow.is_some() {
                    // unwrap rationale: this cannot be triggered if not attached to a window
                    self.ask_where_to_save(
                        sender.input_sender(),
                        &root.toplevel_window().unwrap(),
                        false,
                        FlowInputs::NoOp,
                    );
                }
            }
            FlowInputs::SaveAsFlow => {
                if self.open_flow.is_some() {
                    // unwrap rationale: this cannot be triggered if not attached to a window
                    self.ask_where_to_save(
                        sender.input_sender(),
                        &root.toplevel_window().unwrap(),
                        true,
                        FlowInputs::NoOp,
                    );
                }
            }
            FlowInputs::_SaveFlowThen(then) => {
                // unwrap rationale: this cannot be triggered if not attached to a window
                self.ask_where_to_save(
                    sender.input_sender(),
                    &root.toplevel_window().unwrap(),
                    false,
                    *then,
                );
            }
            FlowInputs::__SaveFlowThen(path, then) => {
                self.open_path = Some(path);
                if let Err(e) = self.save_flow() {
                    self.create_message_dialog(lang::lookup("flow-error-saving"), e.to_string())
                        .set_visible(true);
                } else {
                    widgets
                        .toast_target
                        .add_toast(adw::Toast::new(&lang::lookup("flow-saved")));
                    sender.input_sender().emit(*then);
                }
            }
            FlowInputs::CloseFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_CloseFlow);
            }
            FlowInputs::_CloseFlow => {
                self.close_flow();
            }

            FlowInputs::RunFlow => {
                if let Some(flow) = &self.open_flow {
                    let e_dialog = execution_dialog::ExecutionDialog::builder()
                        .transient_for(root)
                        .launch(execution_dialog::ExecutionDialogInit {
                            flow: flow.clone(),
                            engine_list: self.engine_list.clone(),
                            action_map: self.action_map.clone(),
                        });
                    let dialog = e_dialog.widget();
                    dialog.set_modal(true);
                    dialog.set_visible(true);
                    self.execution_dialog = Some(e_dialog);
                }
            }

            FlowInputs::AddStep(step_id) => {
                if self.open_flow.is_none() {
                    self.new_flow();
                }

                // unwrap rationale: we've just guaranteed a flow is open
                let flow = self.open_flow.as_mut().unwrap();
                // unwrap rationale: the header can't ask to add an action that doesn't exist
                flow.actions.push(ActionConfiguration::from(
                    self.action_map.get_action_by_id(&step_id).unwrap(),
                ));
                self.needs_saving = true;
                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }

            FlowInputs::UpdateStepsFromModel => {
                let mut live_list = self.live_actions_list.guard();
                live_list.clear();
                if let Some(flow) = &self.open_flow {
                    let mut possible_outputs = vec![];
                    for (step, config) in flow.actions.iter().enumerate() {
                        live_list.push_back(action_component::ActionComponentInitialiser {
                            possible_outputs: possible_outputs.clone(),
                            config: config.clone(),
                            action: self.action_map.get_action_by_id(&config.action_id).unwrap(), // rationale: we have already checked the actions are here when the file is opened
                        });
                        // add possible outputs to list AFTER processing this step
                        // unwrap rationale: actions are check to exist prior to opening.
                        for (output_idx, (name, kind)) in self
                            .action_map
                            .get_action_by_id(&config.action_id)
                            .unwrap()
                            .outputs()
                            .iter()
                            .enumerate()
                        {
                            possible_outputs.push((
                                lang::lookup_with_args("source-from-step", {
                                    let mut map = HashMap::new();
                                    map.insert("step", (step + 1).into());
                                    map.insert("name", name.clone().into());
                                    map
                                }),
                                *kind,
                                ActionParameterSource::FromOutput(step, output_idx),
                            ));
                        }
                    }
                }
            }

            FlowInputs::RemoveStep(step_idx) => {
                let idx = step_idx.current_index();
                let flow = self.open_flow.as_mut().unwrap();

                // This is needed as sometimes, if a menu item lines up above the delete step button,
                // they can both be simultaneously triggered.
                if idx >= flow.actions.len() {
                    log::warn!("Skipped running RemoveStep as the index was invalid.");
                    return;
                }

                log::info!("Deleting step {}", idx + 1);

                flow.actions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in flow.actions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => {
                                    *source = ActionParameterSource::Literal
                                }
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                self.needs_saving = true;

                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }
            FlowInputs::CutStep(step_idx) => {
                let idx = step_idx.current_index();
                let flow = self.open_flow.as_mut().unwrap();
                log::info!("Cut step {}", idx + 1);

                // This is needed as sometimes, if a menu item lines up above a button that triggers this,
                // they can both be simultaneously triggered.
                if idx >= flow.actions.len() {
                    log::warn!("Skipped running CutStep as the index was invalid.");
                    return;
                }

                flow.actions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in flow.actions.iter_mut() {
                    for (_param_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => *from_step = usize::MAX,
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                log::debug!("After cut, flow is: {flow:?}");

                self.needs_saving = true;
            }
            FlowInputs::PasteStep(idx, mut config) => {
                let flow = self.open_flow.as_mut().unwrap();
                let idx = idx.max(0).min(flow.actions.len());

                // Adjust step just about to paste
                for (_param_idx, source) in config.parameter_sources.iter_mut() {
                    if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                        if *from_step <= idx {
                            *source = ActionParameterSource::Literal;
                        }
                    }
                }

                log::info!("Pasting step to {}", idx + 1);
                flow.actions.insert(idx, config);

                // Remove references to step and renumber references above step to one less than they were
                for (step_idx, step) in flow.actions.iter_mut().enumerate() {
                    for (_param_idx, source) in step.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(from_step, _output_idx) = source {
                            if *from_step == usize::MAX {
                                if step_idx < idx {
                                    // can't refer to it anymore
                                    *source = ActionParameterSource::Literal;
                                } else {
                                    *from_step = idx;
                                }
                            } else if *from_step >= idx {
                                *from_step += 1;
                            }
                        }
                    }
                }

                log::debug!("After paste, flow is: {flow:?}");

                self.needs_saving = true;

                // Trigger UI steps refresh
                sender.input(FlowInputs::UpdateStepsFromModel);
            }
            FlowInputs::MoveStep(from, to, offset) => {
                let current_from = from.current_index();
                let step = self.open_flow.as_ref().unwrap().actions[current_from].clone();
                sender.input(FlowInputs::CutStep(from));

                // Establish new position
                let mut to = (to.current_index() as isize + offset).max(0) as usize;
                if to > current_from && to > 0 {
                    to -= 1;
                }

                sender.input(FlowInputs::PasteStep(to, step));
            }
        }
        self.update_view(widgets, sender);
    }
}
