use std::{env, fmt::Debug, sync::Arc};

use iced::{settings::Settings, window::icon, Element, Sandbox};
use testangel::*;

mod action_editor;
mod flow_editor;
mod get_started;

pub(crate) fn initialise_ui() {
    let mut settings = Settings::default();
    settings.window.icon = Some(
        icon::from_file_data(include_bytes!("../../../icon.png"), None).expect("icon was invalid!"),
    );
    settings.window.platform_specific.application_id = String::from("TestAngel");
    App::run(settings).expect("Couldn't open UI");
}

#[derive(Default)]
pub struct App {
    state: State,
    action_editor: action_editor::ActionEditor,
    flow_editor: flow_editor::FlowEditor,
    get_started: get_started::GetStarted,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    ActionEditor(action_editor::ActionEditorMessage),
    FlowEditor(flow_editor::FlowEditorMessage),
    GetStarted(get_started::GetStartedMessage),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum State {
    #[default]
    GetStarted,
    AutomationFlowEditor,
    AutomationFlowRunning,
    ActionEditor,
}

impl App {
    fn change_state(&mut self, next_state: State) {
        if next_state == State::AutomationFlowEditor {
            // reload actions
            let _actions_rc = Arc::new(action_loader::get_actions());
        }
        self.state = next_state;
    }
}

impl Sandbox for App {
    type Message = AppMessage;

    fn new() -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let engines_rc = Arc::new(ipc::get_engines());
        let _actions_rc = Arc::new(action_loader::get_actions());
        Self {
            action_editor: action_editor::ActionEditor::new(engines_rc),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        let sub_title = match self.state {
            State::GetStarted => self.get_started.title(),
            State::ActionEditor => self.action_editor.title(),
            State::AutomationFlowEditor => self.flow_editor.title(),
            _ => todo!(),
        };
        let separator = if sub_title.is_some() { " :: " } else { "" };
        let sub_title = sub_title.unwrap_or_default();
        format!("TestAngel{separator}{sub_title}")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            AppMessage::ActionEditor(msg) => {
                if let Some(msg_out) = self.action_editor.update(msg) {
                    match msg_out {
                        action_editor::ActionEditorMessageOut::CloseActionEditor => {
                            self.state = State::GetStarted;
                        }
                    }
                }
            }
            AppMessage::FlowEditor(msg) => {
                self.flow_editor.update(msg);
            }
            AppMessage::GetStarted(msg) => {
                if let Some(msg_out) = self.get_started.update(msg) {
                    match msg_out {
                        get_started::GetStartedMessage::NewAction => {
                            self.state = State::ActionEditor;
                            self.action_editor.new_action();
                        }
                        get_started::GetStartedMessage::NewFlow => {
                            self.state = State::AutomationFlowEditor;
                            self.flow_editor.new_flow();
                        }
                        get_started::GetStartedMessage::OpenAction => {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("TestAngel Actions", &["taaction"])
                                .set_title("Open Action")
                                .set_directory(
                                    env::current_dir()
                                        .expect("Failed to read current directory")
                                        .join("actions"),
                                )
                                .pick_file()
                            {
                                if let Err(e) = self.action_editor.open_action(file) {
                                    rfd::MessageDialog::new()
                                        .set_level(rfd::MessageLevel::Error)
                                        .set_title("Failed to open action")
                                        .set_description(&format!("{e}"))
                                        .set_buttons(rfd::MessageButtons::Ok)
                                        .show();
                                } else {
                                    self.state = State::ActionEditor;
                                }
                            }
                        }
                        get_started::GetStartedMessage::OpenFlow => {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("TestAngel Flows", &["taflow"])
                                .set_title("Open Flow")
                                .set_directory(
                                    env::current_dir().expect("Failed to read current directory"),
                                )
                                .pick_file()
                            {
                                self.state = State::AutomationFlowEditor;
                                self.flow_editor.open_flow(file);
                            }
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Render content
        let content: Element<'_, AppMessage> = match self.state {
            State::GetStarted => self.get_started.view().map(AppMessage::GetStarted),
            State::ActionEditor => self.action_editor.view().map(AppMessage::ActionEditor),
            State::AutomationFlowEditor => self.flow_editor.view().map(AppMessage::FlowEditor),
            _ => todo!(),
        };

        content
    }
}

trait UiComponent {
    type Message: Debug + Send;
    type MessageOut: Debug + Send;

    fn title(&self) -> Option<&str>;

    /// Handle a message.
    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut>;

    /// Render the central panel UI.
    fn view(&self) -> Element<'_, Self::Message>;
}
