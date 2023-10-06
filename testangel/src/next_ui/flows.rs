use std::{fs, path::PathBuf, rc::Rc};

use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent,
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

#[relm4::component(pub)]
impl SimpleComponent for FlowsHeader {
    type Init = ();
    type Input = ();
    type Output = FlowsHeaderOutput;

    view! {
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_icon_name: relm4_icons::icon_name::PAPER,
                set_tooltip: &t!("flows.header.new"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::NewFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::LOUPE,
                set_tooltip: &t!("flows.header.open"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::OpenFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::FLOPPY,
                set_tooltip: &t!("flows.header.save"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::SaveFlow).unwrap();
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
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &t!("flows.header.more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_position: gtk::PositionType::Bottom,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 1,

                        gtk::Button {
                            set_label: "Save flow as...",
                            add_css_class: "flat",

                            connect_clicked[sender] => move |_| {
                                // unwrap rationale: receivers will never be dropped
                                sender.output(FlowsHeaderOutput::SaveAsFlow).unwrap();
                            },
                        },

                        gtk::Button {
                            set_label: "Close flow",
                            add_css_class: "flat",

                            connect_clicked[sender] => move |_| {
                                // unwrap rationale: receivers will never be dropped
                                sender.output(FlowsHeaderOutput::CloseFlow).unwrap();
                            },
                        },
                    },
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

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

#[derive(Debug)]
pub enum FlowInputs {
    /// Do nothing
    NoOp,
    /// Create a new flow
    NewFlow,
    /// Prompt the user to open a flow
    OpenFlow,
    /// Actually open a flow after the user has finished selecting.
    _OpenFlow,
}

#[derive(Debug)]
pub struct FlowsModel {
    open_flow: Option<AutomationFlow>,
    open_path: Option<PathBuf>,
    needs_saving: bool,
    header: Rc<Controller<FlowsHeader>>,

    open_dialog: gtk::FileChooserDialog,
    flow_loading_error_dialog: gtk::MessageDialog,
    flow_actions_changed_dialog: gtk::MessageDialog,
}

impl FlowsModel {
    /// Get an [`Rc`] clone of the header controller
    pub fn header_controller_rc(&self) -> Rc<Controller<FlowsHeader>> {
        self.header.clone()
    }

    /// Create a message dialog attached to the toplevel window.
    fn create_message_dialog<S>(&self, title: S, message: S) -> gtk::MessageDialog
    where
        S: AsRef<str>,
    {
        let dialog = gtk::MessageDialog::builder()
            .transient_for(&self.header.widget().toplevel_window().expect(
                "FlowsModel::create_message_dialog cannot be called until the header is attached",
            ))
            .buttons(gtk::ButtonsType::Ok)
            .title(title.as_ref())
            .text(message.as_ref())
            .build();
        dialog.set_modal(true);
        dialog.connect_response(|d, _| d.close());
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
        Ok(steps_reset)
    }
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsModel {
    type Init = (
        gtk::FileChooserDialog, // A file chooser transient for the parent window
        gtk::MessageDialog, // A message dialog transient for the parent window with an Ok button
        gtk::MessageDialog, // A message dialog transient for the parent window with an Ok button
    );
    type Input = FlowInputs;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,

            gtk::Label {
                set_markup: &t!("flows.nothing-open"),
                #[watch]
                set_visible: model.open_flow.is_none(),
                set_margin_top: 64,
            },
        },

        #[local_ref]
        open_dialog -> gtk::FileChooserDialog {
            set_title: Some(&t!("flows.open")),
            set_action: gtk::FileChooserAction::Open,
            set_modal: true,
            add_button[gtk::ResponseType::Ok]: "Open",

            connect_response[sender] => move |_, response| {
                match response {
                    gtk::ResponseType::Ok => sender.input(FlowInputs::_OpenFlow),
                    _ => (),
                }
            },
        },

        #[local_ref]
        flow_loading_error_dialog -> gtk::MessageDialog {
            set_title: Some(&t!("flows.error-opening")),
            set_modal: true,

            connect_response => move |dialog, response| {
                log::info!("response type: {response}");
                match response {
                    gtk::ResponseType::Ok => dialog.close(),
                    _ => (),
                }
            },
        },

        #[local_ref]
        flow_actions_changed_dialog -> gtk::MessageDialog {
            set_title: Some(&t!("flows.action-changed")),
            set_modal: true,

            connect_response => move |dialog, response| {
                match response {
                    gtk::ResponseType::Ok => dialog.close(),
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
                FlowsHeaderOutput::SaveFlow => FlowInputs::NoOp,
                FlowsHeaderOutput::SaveAsFlow => FlowInputs::NoOp,
                FlowsHeaderOutput::CloseFlow => FlowInputs::NoOp,
                FlowsHeaderOutput::RunFlow => FlowInputs::NoOp,
            },
        ));

        let model = FlowsModel {
            open_flow: None,
            open_path: None,
            needs_saving: false,
            open_dialog: init.0,
            flow_loading_error_dialog: init.1,
            flow_actions_changed_dialog: init.2,
            header,
        };

        let open_dialog = &model.open_dialog;
        let flow_loading_error_dialog = &model.flow_loading_error_dialog;
        let flow_actions_changed_dialog = &model.flow_actions_changed_dialog;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            FlowInputs::NoOp => (),
            FlowInputs::NewFlow => {
                if self.needs_saving {
                    // TODO Prompt to save before close
                }

                self.open_path = None;
                self.needs_saving = false;
                self.open_flow = Some(AutomationFlow::default());
            }
            FlowInputs::OpenFlow => {
                self.open_dialog.show();
            }
            FlowInputs::_OpenFlow => {
                self.open_dialog.hide();

                if let Some(file) = self.open_dialog.file() {
                    if let Some(path) = file.path() {
                        match self.open_flow(path) {
                            Ok(changes) => {
                                if !changes.is_empty() {
                                    // TODO Show dialog of changes
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
        }
    }
}
