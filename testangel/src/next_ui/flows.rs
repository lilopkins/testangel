use std::{fs, path::PathBuf, rc::Rc};

use adw::prelude::*;
use gtk::prelude::*;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent, actions::{RelmAction, RelmActionGroup, AccelsPlus},
};
use rust_i18n::t;
use testangel::types::{AutomationFlow, VersionedFile};

mod action_component;

#[derive(Debug)]
pub struct FlowsHeader;

#[derive(Debug)]
pub enum FlowsHeaderOutput {
    NewFlow,
    OpenFlow,
    SaveFlow,
    SaveAsFlow,
    CloseFlow,
    RunFlow,
}

#[derive(Debug)]
pub enum FlowsHeaderInput {
    OpenAboutDialog,
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsHeader {
    type Init = ();
    type Input = FlowsHeaderInput;
    type Output = FlowsHeaderOutput;

    view! {
        #[root]
        #[name = "start"]
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_label: &t!("open"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::OpenFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::PLAY,
                set_tooltip: &t!("flows.header.run"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::RunFlow).unwrap();
                },
            },
        },

        #[name = "end"]
        gtk::Box {
            set_spacing: 5,
            
            gtk::Button {
                set_label: &t!("save"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::SaveFlow).unwrap();
                },
            },
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &t!("flows.header.more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&flows_menu)) {
                    set_position: gtk::PositionType::Bottom,
                }
            },
        },
    }

    menu! {
        flows_menu: {
            &t!("flows.header.new") => FlowsNewAction,
            &t!("flows.header.save-as") => FlowsSaveAsAction,
            &t!("flows.header.close") => FlowsCloseAction,
            section! {
                &t!("header.about") => FlowsAboutAction,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader;
        let widgets = view_output!();

        let about_action: RelmAction<FlowsAboutAction> = RelmAction::new_stateless(move |_| {
            sender.input(FlowsHeaderInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<FlowsAboutAction>(&["<primary>A"]);

        let mut group = RelmActionGroup::<FlowsActionGroup>::new();
        group.add_action(about_action);
        group.register_for_widget(&widgets.end);

        ComponentParts { model, widgets }
    }
    
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowsHeaderInput::OpenAboutDialog => {
                super::about::AppAbout::builder()
                    // .transient_for()
                    .launch(())
                    .widget()
                    .show();
            }
        }
    }
}

relm4::new_action_group!(FlowsActionGroup, "flows");
relm4::new_stateless_action!(FlowsNewAction, FlowsActionGroup, "new");
relm4::new_stateless_action!(FlowsSaveAsAction, FlowsActionGroup, "save-as");
relm4::new_stateless_action!(FlowsCloseAction, FlowsActionGroup, "close");
relm4::new_stateless_action!(FlowsAboutAction, FlowsActionGroup, "about");

pub enum SaveOrOpenFlowError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    FlowNotVersionCompatible,
    MissingAction(String),
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
            Self::MissingAction(e) => t!("flows.save-or-open-flow-error.missing-action", error = e),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FlowInputs {
    /// Do nothing
    NoOp,
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
    _CloseFlow,
}

#[derive(Debug)]
pub struct FlowsModel {
    open_flow: Option<AutomationFlow>,
    open_path: Option<PathBuf>,
    needs_saving: bool,
    header: Rc<Controller<FlowsHeader>>,

    open_dialog: gtk::FileChooserDialog,
    save_dialog: gtk::FileChooserDialog,
}

impl FlowsModel {
    /// Get an [`Rc`] clone of the header controller
    pub fn header_controller_rc(&self) -> Rc<Controller<FlowsHeader>> {
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
            // TODO Check for missing action
            // match self.actions_list.get_action_by_id(&ac.action_id) {
            //     None => return Err(SaveOrOpenFlowError::MissingAction(ac.action_id.clone())),
            //     Some(action) => {
            //         // Check that action parameters haven't changed. If they have, reset values.
            //         if ac.update(action) {
            //             steps_reset.push(step + 1);
            //         }
            //     }
            // }
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
    fn ask_where_to_save(&mut self, sender: &relm4::Sender<FlowInputs>, always_ask_where: bool, then: FlowInputs) {
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
    );
    type Input = FlowInputs;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,

            adw::StatusPage {
                set_title: &t!("flows.nothing-open"),
                set_description: Some(&t!("flows.nothing-open-description")),
                set_icon_name: Some(relm4_icons::icon_name::LIGHTBULB),
                #[watch]
                set_visible: model.open_flow.is_none(),
                set_vexpand: true,
            },
        },

        #[local_ref]
        open_dialog -> gtk::FileChooserDialog {
            set_title: Some(&t!("flows.open")),
            set_action: gtk::FileChooserAction::Open,
            set_modal: true,
            add_button[gtk::ResponseType::Ok]: &t!("open"),

            connect_response[sender] => move |_, response| {
                match response {
                    gtk::ResponseType::Ok => sender.input(FlowInputs::__OpenFlow),
                    _ => (),
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
                match response {
                    gtk::ResponseType::Ok => sender.input(FlowInputs::_SaveFlowThen(Box::new(FlowInputs::NoOp))),
                    _ => (),
                }
            },
        },
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Rc::new(FlowsHeader::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                FlowsHeaderOutput::NewFlow => FlowInputs::NewFlow,
                FlowsHeaderOutput::OpenFlow => FlowInputs::OpenFlow,
                FlowsHeaderOutput::SaveFlow => FlowInputs::SaveFlow,
                FlowsHeaderOutput::SaveAsFlow => FlowInputs::SaveAsFlow,
                FlowsHeaderOutput::CloseFlow => FlowInputs::CloseFlow,
                FlowsHeaderOutput::RunFlow => FlowInputs::NoOp,
            },
        ));

        let model = FlowsModel {
            open_flow: None,
            open_path: None,
            needs_saving: false,
            open_dialog: init.0,
            save_dialog: init.1,
            header,
        };

        let open_dialog = &model.open_dialog;
        let save_dialog = &model.save_dialog;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowInputs::NoOp => (),
            FlowInputs::NewFlow => {
                self.prompt_to_save(sender.input_sender(), FlowInputs::_NewFlow);
            }
            FlowInputs::_NewFlow => {
                self.open_path = None;
                self.needs_saving = true;
                self.open_flow = Some(AutomationFlow::default());
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
        }
    }
}
