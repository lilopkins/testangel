use std::{cmp::Ordering, collections::HashMap, fs, path::PathBuf, rc::Rc, sync::Arc};

use adw::prelude::*;
use relm4::{
    adw, component::Connector, factory::FactoryVecDeque, gtk, prelude::DynamicIndex, Component,
    ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
};
use testangel::{
    action_loader::ActionMap,
    ipc::EngineList,
    types::{Action, InstructionConfiguration, InstructionParameterSource, VersionedFile},
};
use testangel_ipc::prelude::ParameterKind;

use super::{file_filters, lang};

mod execution_dialog;
pub mod header;
mod instruction_component;
mod metadata_component;
mod params;

pub enum SaveOrOpenActionError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    ActionNotVersionCompatible,
    MissingInstruction(usize, String),
}

impl ToString for SaveOrOpenActionError {
    fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => lang::lookup_with_args("action-save-open-error-io-error", {
                let mut map = HashMap::new();
                map.insert("error", e.to_string().into());
                map
            }),
            Self::ParsingError(e) => {
                lang::lookup_with_args("action-save-open-error-parsing-error", {
                    let mut map = HashMap::new();
                    map.insert("error", e.to_string().into());
                    map
                })
            }
            Self::SerializingError(e) => {
                lang::lookup_with_args("action-save-open-error-serializing-error", {
                    let mut map = HashMap::new();
                    map.insert("error", e.to_string().into());
                    map
                })
            }
            Self::ActionNotVersionCompatible => {
                lang::lookup("action-save-open-error-action-not-version-compatible")
            }
            Self::MissingInstruction(step, e) => {
                lang::lookup_with_args("action-save-open-error-missing-instruction", {
                    let mut map = HashMap::new();
                    map.insert("step", (step + 1).into());
                    map.insert("error", e.to_string().into());
                    map
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum ActionInputs {
    /// Do nothing
    NoOp,
    /// The map of actions has changed and should be updated
    ActionsMapChanged(Arc<ActionMap>),
    /// Create a new action
    NewAction,
    /// Actually create the new action
    _NewAction,
    /// Prompt the user to open an action. This will ask to save first if needed.
    OpenAction,
    /// Actually show the user the open file dialog
    _OpenAction,
    /// Actually open an action after the user has finished selecting
    __OpenAction(PathBuf),
    /// Save the action, prompting if needed to set file path
    SaveAction,
    /// Save the action as a new file, always prompting for a file path
    SaveAsAction,
    /// Ask where to save if needed, then save
    _SaveActionThen(Box<ActionInputs>),
    /// Actually write the action to disk, then emit then input
    __SaveActionThen(PathBuf, Box<ActionInputs>),
    /// Close the action, prompting if needing to save first
    CloseAction,
    /// Actually close the action
    _CloseAction,
    /// Add the step with the ID provided
    AddStep(String),
    /// Update the UI steps from the open action. This will clear first and overwrite any changes!
    UpdateStepsFromModel,
    /// Remove the step with the provided index, resetting all references to it.
    RemoveStep(DynamicIndex),
    /// Remove the step with the provided index, but change references to it to a temporary value (`usize::MAX`)
    /// that can be set again with [`ActionInputs::PasteStep`].
    /// This doesn't refresh the UI until Paste is called.
    CutStep(DynamicIndex),
    /// Insert a step at the specified index and set references back to the correct step.
    /// This refreshes the UI.
    PasteStep(usize, InstructionConfiguration),
    /// Move a step from the index to a position offset (param 3) from a new index (param 2).
    MoveStep(DynamicIndex, DynamicIndex, isize),
    /// Show the action test dialog
    RunAction,
    /// The [`InstructionConfiguration`] has changed for the step indicated by the [`DynamicIndex`].
    /// This does not refresh the UI.
    ConfigUpdate(DynamicIndex, InstructionConfiguration),
    /// The metadata has been updated and the action should be updated to reflect that
    MetadataUpdated(metadata_component::MetadataOutput),
    /// Set parameters
    SetParameters(Vec<(String, ParameterKind)>),
    /// Remove references to the provided index, or reduce any higher than.
    ParamIndexRemoved(usize),
    /// Swap references to the indexes provided
    ParamIndexesSwapped(usize, usize),
}
#[derive(Clone, Debug)]
pub enum ActionOutputs {
    /// Inform other parts that actions may have changed, reload them!
    ReloadActions,
}

#[derive(Debug)]
pub struct ActionsModel {
    action_map: Arc<ActionMap>,
    engine_list: Arc<EngineList>,

    open_action: Option<Action>,
    open_path: Option<PathBuf>,
    needs_saving: bool,
    header: Rc<Controller<header::ActionsHeader>>,
    metadata: Controller<metadata_component::Metadata>,
    parameters: Controller<params::ActionParams>,
    live_instructions_list: FactoryVecDeque<instruction_component::InstructionComponent>,

    execution_dialog: Option<Connector<execution_dialog::ExecutionDialog>>,
}

impl ActionsModel {
    /// Get an [`Rc`] clone of the header controller
    pub fn header_controller_rc(&self) -> Rc<Controller<header::ActionsHeader>> {
        self.header.clone()
    }

    /// Create the absolute barebones of a message dialog, allowing for custom button and response mapping.
    fn create_message_dialog_skeleton<S>(&self, title: S, message: S) -> adw::MessageDialog
    where
        S: AsRef<str>,
    {
        adw::MessageDialog::builder()
            .transient_for(&self.header.widget().toplevel_window().expect(
                "ActionsModel::create_message_dialog cannot be called until the header is attached",
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

    /// Just open a brand new action
    fn new_action(&mut self) {
        self.open_path = None;
        self.needs_saving = true;
        self.open_action = Some(Action::default());
        self.header
            .emit(header::ActionsHeaderInput::ChangeActionOpen(
                self.open_action.is_some(),
            ));
        self.metadata
            .emit(metadata_component::MetadataInput::ChangeAction(
                Action::default(),
            ));
        self.parameters
            .emit(params::ActionParamsInput::ChangeAction(Action::default()));
    }

    /// Open an action. This does not ask to save first.
    fn open_action(&mut self, file: PathBuf) -> Result<(), SaveOrOpenActionError> {
        let data = &fs::read_to_string(&file).map_err(SaveOrOpenActionError::IoError)?;

        let versioned_file: VersionedFile =
            ron::from_str(data).map_err(SaveOrOpenActionError::ParsingError)?;
        if versioned_file.version() != 1 {
            return Err(SaveOrOpenActionError::ActionNotVersionCompatible);
        }

        let mut action: Action =
            ron::from_str(data).map_err(SaveOrOpenActionError::ParsingError)?;
        if action.version() != 1 {
            return Err(SaveOrOpenActionError::ActionNotVersionCompatible);
        }
        for (step, ic) in action.instructions.iter_mut().enumerate() {
            if self
                .engine_list
                .get_engine_by_instruction_id(&ic.instruction_id)
                .is_none()
            {
                return Err(SaveOrOpenActionError::MissingInstruction(
                    step,
                    ic.instruction_id.clone(),
                ));
            }
        }

        self.open_action = Some(action.clone());
        self.header
            .emit(header::ActionsHeaderInput::ChangeActionOpen(
                self.open_action.is_some(),
            ));
        self.metadata
            .emit(metadata_component::MetadataInput::ChangeAction(
                action.clone(),
            ));
        self.parameters
            .emit(params::ActionParamsInput::ChangeAction(action));
        self.open_path = Some(file);
        self.needs_saving = false;
        log::debug!("New action open.");
        log::debug!("Action: {:?}", self.open_action);
        Ok(())
    }

    /// Ask the user if they want to save this file. If they response yes, this will also trigger the save function.
    /// This function will only ask the user if needed, otherwise it will emit immediately.
    fn prompt_to_save(&self, sender: &relm4::Sender<ActionInputs>, then: ActionInputs) {
        if self.needs_saving {
            let question = self.create_message_dialog_skeleton(
                lang::lookup("action-save-before"),
                lang::lookup("action-save-before-message"),
            );
            question.add_response("discard", &lang::lookup("discard"));
            question.add_response("save", &lang::lookup("save"));
            question.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
            question.set_default_response(Some("save"));
            question.set_close_response("discard");
            let sender_c = sender.clone();
            let then_c = then.clone();
            question.connect_response(Some("save"), move |_, _| {
                sender_c.emit(ActionInputs::_SaveActionThen(Box::new(then_c.clone())));
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
        sender: &relm4::Sender<ActionInputs>,
        transient_for: &impl IsA<gtk::Window>,
        always_ask_where: bool,
        then: ActionInputs,
    ) {
        if always_ask_where || self.open_path.is_none() {
            // Ask where
            let dialog = gtk::FileDialog::builder()
                .modal(true)
                .title(lang::lookup("action-header-save"))
                .initial_folder(&gtk::gio::File::for_path(
                    std::env::var("TA_ACTION_DIR").unwrap_or("./actions".to_string()),
                ))
                .filters(&file_filters::filter_list(vec![
                    file_filters::actions(),
                    file_filters::all(),
                ]))
                .build();

            let sender_c = sender.clone();
            dialog.save(
                Some(transient_for),
                Some(&relm4::gtk::gio::Cancellable::new()),
                move |res| {
                    if let Ok(file) = res {
                        let path = file.path().unwrap();
                        sender_c.emit(ActionInputs::__SaveActionThen(path, Box::new(then.clone())));
                    }
                },
            );
        } else {
            sender.emit(ActionInputs::__SaveActionThen(
                self.open_path.clone().unwrap(),
                Box::new(then),
            ));
        }
    }

    /// Just save the action to disk with the current `open_path` as the destination
    fn save_action(&mut self) -> Result<(), SaveOrOpenActionError> {
        let save_path = self.open_path.as_ref().unwrap();
        let data = ron::to_string(self.open_action.as_ref().unwrap())
            .map_err(SaveOrOpenActionError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenActionError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Close this action without checking first
    fn close_action(&mut self) {
        self.open_action = None;
        self.open_path = None;
        self.needs_saving = false;
        self.live_instructions_list.guard().clear();
        self.header
            .emit(header::ActionsHeaderInput::ChangeActionOpen(
                self.open_action.is_some(),
            ));
    }
}

#[relm4::component(pub)]
impl Component for ActionsModel {
    type Init = (Arc<ActionMap>, Arc<EngineList>);
    type Input = ActionInputs;
    type Output = ActionOutputs;
    type CommandOutput = ();

    view! {
        #[root]
        toast_target = adw::ToastOverlay {
            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                if model.open_action.is_none() {
                    adw::StatusPage {
                        set_title: &lang::lookup("nothing-open"),
                        set_description: Some(&lang::lookup("action-nothing-open-description")),
                        set_icon_name: Some(relm4_icons::icon_name::LIGHTBULB),
                        set_vexpand: true,
                    }
                } else {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 10,
                        set_spacing: 10,

                        model.metadata.widget(),

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                        },

                        model.parameters.widget(),

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                        },

                        #[local_ref]
                        live_instructions_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        },
                    }
                }
            },
        },
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Rc::new(
            header::ActionsHeader::builder()
                .launch((init.1.clone(), init.0.clone()))
                .forward(sender.input_sender(), |msg| match msg {
                    header::ActionsHeaderOutput::NewAction => ActionInputs::NewAction,
                    header::ActionsHeaderOutput::OpenAction => ActionInputs::OpenAction,
                    header::ActionsHeaderOutput::SaveAction => ActionInputs::SaveAction,
                    header::ActionsHeaderOutput::SaveAsAction => ActionInputs::SaveAsAction,
                    header::ActionsHeaderOutput::CloseAction => ActionInputs::CloseAction,
                    header::ActionsHeaderOutput::RunAction => ActionInputs::RunAction,
                    header::ActionsHeaderOutput::AddStep(step) => ActionInputs::AddStep(step),
                }),
        );

        let model = ActionsModel {
            action_map: init.0,
            engine_list: init.1,
            open_action: None,
            open_path: None,
            needs_saving: false,
            execution_dialog: None,
            header,
            live_instructions_list: FactoryVecDeque::new(
                gtk::Box::default(),
                sender.input_sender(),
            ),
            metadata: metadata_component::Metadata::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| {
                    ActionInputs::MetadataUpdated(msg)
                }),
            parameters: params::ActionParams::builder().launch(()).forward(
                sender.input_sender(),
                |msg| match msg {
                    params::ActionParamsOutput::IndexRemoved(idx) => {
                        ActionInputs::ParamIndexRemoved(idx)
                    }
                    params::ActionParamsOutput::IndexesSwapped(a, b) => {
                        ActionInputs::ParamIndexesSwapped(a, b)
                    }
                    params::ActionParamsOutput::SetParameters(new_params) => {
                        ActionInputs::SetParameters(new_params)
                    }
                },
            ),
        };

        // Trigger update actions from model
        sender.input(ActionInputs::UpdateStepsFromModel);

        let live_instructions_list = model.live_instructions_list.widget();
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
            ActionInputs::NoOp => (),

            ActionInputs::MetadataUpdated(meta) => {
                if let Some(action) = self.open_action.as_mut() {
                    if let Some(new_name) = meta.new_name {
                        action.friendly_name = new_name;
                    }
                    if let Some(new_group) = meta.new_group {
                        action.group = new_group;
                    }
                    if let Some(new_author) = meta.new_author {
                        action.author = new_author;
                    }
                    if let Some(new_description) = meta.new_description {
                        action.description = new_description;
                    }
                    if let Some(new_visible) = meta.new_visible {
                        action.visible = new_visible;
                    }
                }
            }

            ActionInputs::SetParameters(new_params) => {
                if let Some(action) = self.open_action.as_mut() {
                    action.parameters = new_params;
                    sender.input(ActionInputs::UpdateStepsFromModel);
                }
            }
            ActionInputs::ParamIndexRemoved(idx) => {
                if let Some(action) = self.open_action.as_mut() {
                    for ic in action.instructions.iter_mut() {
                        for (_, src) in ic.parameter_sources.iter_mut() {
                            if let InstructionParameterSource::FromParameter(n) = src {
                                match idx.cmp(n) {
                                    Ordering::Equal => *src = InstructionParameterSource::Literal,
                                    Ordering::Less => *n -= 1,
                                    _ => (),
                                }
                            }
                        }
                    }
                    sender.input(ActionInputs::UpdateStepsFromModel);
                }
            }
            ActionInputs::ParamIndexesSwapped(a, b) => {
                if let Some(action) = self.open_action.as_mut() {
                    for ic in action.instructions.iter_mut() {
                        for (_, src) in ic.parameter_sources.iter_mut() {
                            if let InstructionParameterSource::FromParameter(n) = src {
                                if *n == a {
                                    *n = b;
                                } else if *n == b {
                                    *n = a;
                                }
                            }
                        }
                    }
                    sender.input(ActionInputs::UpdateStepsFromModel);
                }
            }

            ActionInputs::ActionsMapChanged(new_map) => {
                self.action_map = new_map.clone();
                self.header
                    .emit(header::ActionsHeaderInput::ActionsMapChanged(new_map));
            }
            ActionInputs::ConfigUpdate(step, new_config) => {
                // unwrap rationale: config updates can't happen if nothing is open
                let action = self.open_action.as_mut().unwrap();
                action.instructions[step.current_index()] = new_config;
            }
            ActionInputs::NewAction => {
                self.prompt_to_save(sender.input_sender(), ActionInputs::_NewAction);
            }
            ActionInputs::_NewAction => {
                self.new_action();
            }
            ActionInputs::OpenAction => {
                self.prompt_to_save(sender.input_sender(), ActionInputs::_OpenAction);
            }
            ActionInputs::_OpenAction => {
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("action-header-open"))
                    .filters(&file_filters::filter_list(vec![
                        file_filters::actions(),
                        file_filters::all(),
                    ]))
                    .initial_folder(&gtk::gio::File::for_path(
                        std::env::var("TA_ACTION_DIR").unwrap_or("./actions".to_string()),
                    ))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            sender_c.input(ActionInputs::__OpenAction(path));
                        }
                    },
                );
            }
            ActionInputs::__OpenAction(path) => {
                match self.open_action(path) {
                    Ok(_) => {
                        // Reload UI
                        sender.input(ActionInputs::UpdateStepsFromModel);
                    }
                    Err(e) => {
                        // Show error dialog
                        self.create_message_dialog(
                            lang::lookup("action-error-opening"),
                            e.to_string(),
                        )
                        .set_visible(true);
                    }
                }
            }
            ActionInputs::SaveAction => {
                if self.open_action.is_some() {
                    // unwrap rationale: this cannot be triggered if not attached to a window
                    self.ask_where_to_save(
                        sender.input_sender(),
                        &root.toplevel_window().unwrap(),
                        false,
                        ActionInputs::NoOp,
                    );
                }
            }
            ActionInputs::SaveAsAction => {
                if self.open_action.is_some() {
                    // unwrap rationale: this cannot be triggered if not attached to a window
                    self.ask_where_to_save(
                        sender.input_sender(),
                        &root.toplevel_window().unwrap(),
                        true,
                        ActionInputs::NoOp,
                    );
                }
            }
            ActionInputs::_SaveActionThen(then) => {
                // unwrap rationale: this cannot be triggered if not attached to a window
                self.ask_where_to_save(
                    sender.input_sender(),
                    &root.toplevel_window().unwrap(),
                    false,
                    *then,
                );
            }
            ActionInputs::__SaveActionThen(path, then) => {
                self.open_path = Some(path);
                if let Err(e) = self.save_action() {
                    self.create_message_dialog(lang::lookup("action-error-saving"), e.to_string())
                        .set_visible(true);
                } else {
                    widgets
                        .toast_target
                        .add_toast(adw::Toast::new(&lang::lookup("action-saved")));
                    sender.input_sender().emit(*then);
                }
                let _ = sender.output(ActionOutputs::ReloadActions);
            }
            ActionInputs::CloseAction => {
                self.prompt_to_save(sender.input_sender(), ActionInputs::_CloseAction);
            }
            ActionInputs::_CloseAction => {
                self.close_action();
            }

            ActionInputs::RunAction => {
                if let Some(action) = &self.open_action {
                    let e_dialog = execution_dialog::ExecutionDialog::builder()
                        .transient_for(root)
                        .launch(execution_dialog::ExecutionDialogInit {
                            action: action.clone(),
                            engine_list: self.engine_list.clone(),
                            action_map: self.action_map.clone(),
                        });
                    let dialog = e_dialog.widget();
                    dialog.set_modal(true);
                    dialog.set_visible(true);
                    self.execution_dialog = Some(e_dialog);
                }
            }

            ActionInputs::AddStep(step_id) => {
                if self.open_action.is_none() {
                    self.new_action();
                }

                // unwrap rationale: we've just guaranteed a flow is open
                let action = self.open_action.as_mut().unwrap();
                // unwrap rationale: the header can't ask to add an action that doesn't exist
                action.instructions.push(InstructionConfiguration::from(
                    self.engine_list.get_instruction_by_id(&step_id).unwrap(),
                ));
                // Trigger UI steps refresh
                sender.input(ActionInputs::UpdateStepsFromModel);
            }

            ActionInputs::UpdateStepsFromModel => {
                let mut live_list = self.live_instructions_list.guard();
                live_list.clear();
                if let Some(action) = &self.open_action {
                    let mut possible_outputs = vec![];
                    // Populate possible outputs with parameters
                    for (idx, (name, kind)) in action.parameters.iter().enumerate() {
                        possible_outputs.push((
                            lang::lookup_with_args("source-from-param", {
                                let mut map = HashMap::new();
                                map.insert("param", name.clone().into());
                                map
                            }),
                            *kind,
                            InstructionParameterSource::FromParameter(idx),
                        ));
                    }

                    for (step, config) in action.instructions.iter().enumerate() {
                        live_list.push_back(
                            instruction_component::InstructionComponentInitialiser {
                                possible_outputs: possible_outputs.clone(),
                                config: config.clone(),
                                instruction: self
                                    .engine_list
                                    .get_instruction_by_id(&config.instruction_id)
                                    .unwrap(), // rationale: we have already checked the actions are here when the file is opened
                            },
                        );
                        // add possible outputs to list AFTER processing this step
                        // unwrap rationale: actions are check to exist prior to opening.
                        for (output_id, (name, kind)) in self
                            .engine_list
                            .get_instruction_by_id(&config.instruction_id)
                            .unwrap()
                            .outputs()
                            .iter()
                        {
                            possible_outputs.push((
                                lang::lookup_with_args("source-from-step", {
                                    let mut map = HashMap::new();
                                    map.insert("step", (step + 1).into());
                                    map.insert("name", name.clone().into());
                                    map
                                }),
                                *kind,
                                InstructionParameterSource::FromOutput(step, output_id.clone()),
                            ));
                        }
                    }
                }
            }

            ActionInputs::RemoveStep(step_idx) => {
                let idx = step_idx.current_index();
                let action = self.open_action.as_mut().unwrap();

                // This is needed as sometimes, if a menu item lines up above the delete step button,
                // they can both be simultaneously triggered.
                if idx >= action.instructions.len() {
                    log::warn!("Skipped running RemoveStep as the index was invalid.");
                    return;
                }

                log::info!("Deleting step {}", idx + 1);

                action.instructions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in action.instructions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromOutput(from_step, _output_idx) =
                            source
                        {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => {
                                    *source = InstructionParameterSource::Literal
                                }
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                // Trigger UI steps refresh
                sender.input(ActionInputs::UpdateStepsFromModel);
            }
            ActionInputs::CutStep(step_idx) => {
                let idx = step_idx.current_index();
                let action = self.open_action.as_mut().unwrap();
                log::info!("Cut step {}", idx + 1);

                // This is needed as sometimes, if a menu item lines up above a button that triggers this,
                // they can both be simultaneously triggered.
                if idx >= action.instructions.len() {
                    log::warn!("Skipped running CutStep as the index was invalid.");
                    return;
                }

                action.instructions.remove(idx);

                // Remove references to step and renumber references above step to one less than they were
                for step in action.instructions.iter_mut() {
                    for (_step_idx, source) in step.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromOutput(from_step, _output_idx) =
                            source
                        {
                            match (*from_step).cmp(&idx) {
                                std::cmp::Ordering::Equal => *from_step = usize::MAX,
                                std::cmp::Ordering::Greater => *from_step -= 1,
                                _ => (),
                            }
                        }
                    }
                }
            }
            ActionInputs::PasteStep(idx, config) => {
                let action = self.open_action.as_mut().unwrap();
                let idx = idx.max(0).min(action.instructions.len());
                log::info!("Pasting step to {}", idx + 1);
                action.instructions.insert(idx, config);

                // Remove references to step and renumber references above step to one less than they were
                for (step_idx, step) in action.instructions.iter_mut().enumerate() {
                    for (_param_idx, source) in step.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromOutput(from_step, _output_idx) =
                            source
                        {
                            if *from_step == usize::MAX {
                                if step_idx < idx {
                                    // can't refer to it anymore
                                    *source = InstructionParameterSource::Literal;
                                } else {
                                    *from_step = idx;
                                }
                            } else if *from_step >= idx {
                                *from_step += 1;
                            }
                        }
                    }
                }

                // Trigger UI steps refresh
                sender.input(ActionInputs::UpdateStepsFromModel);
            }
            ActionInputs::MoveStep(from, to, offset) => {
                let current_from = from.current_index();
                let step = self.open_action.as_ref().unwrap().instructions[current_from].clone();
                sender.input(ActionInputs::CutStep(from));
                let mut to = (to.current_index() as isize + offset).max(0) as usize;
                if to > current_from && to > 0 {
                    to -= 1;
                }
                sender.input(ActionInputs::PasteStep(to, step));
            }
        }
        self.update_view(widgets, sender);
    }
}
