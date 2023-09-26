use std::{env, fmt::Debug, path::PathBuf, sync::Arc};

use iced::{
    executor,
    settings::Settings,
    window::{self, icon},
    Application, Command, Element, Event, Subscription, Theme,
};
use testangel::{ipc::EngineList, *};

mod action_editor;
mod flow_editor;
mod flow_running;
mod get_started;

pub(crate) fn initialise_ui() {
    let mut settings = Settings::default();
    settings.window.icon = Some(
        icon::from_file_data(include_bytes!("../../../icon.png"), None).expect("icon was invalid!"),
    );
    settings.exit_on_close_request = false;
    #[cfg(target_os = "linux")]
    {
        settings.window.platform_specific.application_id = String::from("TestAngel");
    }
    App::run(settings).expect("Couldn't open UI");
}

#[derive(Default)]
pub struct App {
    engine_list: Arc<EngineList>,

    state: State,
    action_editor: action_editor::ActionEditor,
    flow_editor: flow_editor::FlowEditor,
    flow_running: flow_running::FlowRunning,
    get_started: get_started::GetStarted,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    Event(iced::Event),
    ActionEditor(action_editor::ActionEditorMessage),
    FlowEditor(flow_editor::FlowEditorMessage),
    FlowRunning(flow_running::FlowRunningMessage),
    GetStarted(get_started::GetStartedMessage),
    OpenAction(Option<PathBuf>),
    OpenFlow(Option<PathBuf>),
    CloseEditor,
    NoOp,
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
    fn update_action_list(&mut self) {
        let actions_rc = Arc::new(action_loader::get_actions(self.engine_list.clone()));
        self.flow_editor.update_action_map(actions_rc.clone());
        self.flow_running.update_action_map(actions_rc);
    }
}

impl Application for App {
    type Message = AppMessage;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let engines_rc = Arc::new(ipc::get_engines());
        let actions_rc = Arc::new(action_loader::get_actions(engines_rc.clone()));
        (
            Self {
                engine_list: engines_rc.clone(),
                action_editor: action_editor::ActionEditor::new(engines_rc.clone()),
                flow_editor: flow_editor::FlowEditor::new(actions_rc.clone()),
                flow_running: flow_running::FlowRunning::new(actions_rc, engines_rc),
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let sub_title = match self.state {
            State::GetStarted => self.get_started.title(),
            State::ActionEditor => self.action_editor.title(),
            State::AutomationFlowEditor => self.flow_editor.title(),
            State::AutomationFlowRunning => self.flow_running.title(),
        };
        let separator = if sub_title.is_some() { " :: " } else { "" };
        let sub_title = sub_title.unwrap_or_default();
        format!("TestAngel{separator}{sub_title}")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            match self.state {
                State::GetStarted => self.get_started.subscription().map(AppMessage::GetStarted),
                State::ActionEditor => self
                    .action_editor
                    .subscription()
                    .map(AppMessage::ActionEditor),
                State::AutomationFlowEditor => {
                    self.flow_editor.subscription().map(AppMessage::FlowEditor)
                }
                State::AutomationFlowRunning => self
                    .flow_running
                    .subscription()
                    .map(AppMessage::FlowRunning),
            },
            iced::subscription::events().map(AppMessage::Event),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMessage::NoOp => (),
            AppMessage::Event(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    std::process::exit(0);
                }
            }
            AppMessage::ActionEditor(msg) => {
                let (msg_out, cmd) = self.action_editor.update(msg);
                if let Some(msg_out) = msg_out {
                    match msg_out {

                    }
                }
                if let Some(cmd) = cmd {
                    return cmd;
                }
            }
            AppMessage::FlowEditor(msg) => {
                let (msg_out, cmd) = self.flow_editor.update(msg);
                if let Some(msg_out) = msg_out {
                    match msg_out {
                        flow_editor::FlowEditorMessageOut::RunFlow(flow) => {
                            self.state = State::AutomationFlowRunning;
                            self.flow_running.start_flow(flow);
                        }
                    }
                }
                if let Some(cmd) = cmd {
                    return cmd;
                }
            }
            AppMessage::CloseEditor => {
                self.state = State::GetStarted;
            }
            AppMessage::FlowRunning(msg) => {
                let (msg_out, cmd) = self.flow_running.update(msg);
                if let Some(msg_out) = msg_out {
                    match msg_out {
                        flow_running::FlowRunningMessageOut::BackToEditor => {
                            self.state = State::AutomationFlowEditor;
                        }
                        flow_running::FlowRunningMessageOut::SaveFlowReport(evidence) => {
                            return Command::perform(
                                rfd::AsyncFileDialog::new()
                                    .add_filter("Portable Document Format", &["pdf"])
                                    .set_file_name("report.pdf")
                                    .set_title("Save Report")
                                    .set_directory(env::current_dir().expect("Failed to get cwd"))
                                    .save_file(),
                                |f| {
                                    AppMessage::FlowRunning(flow_running::FlowRunningMessage::Save(
                                        f.map(|f| f.path().to_path_buf()),
                                        evidence,
                                    ))
                                },
                            );
                        }
                    }
                }
                if let Some(cmd) = cmd {
                    return cmd;
                }
            }
            AppMessage::GetStarted(msg) => {
                let (msg_out, cmd) = self.get_started.update(msg);
                if let Some(msg_out) = msg_out {
                    match msg_out {
                        get_started::GetStartedMessage::NewAction => {
                            self.state = State::ActionEditor;
                            self.action_editor.new_action();
                        }
                        get_started::GetStartedMessage::NewFlow => {
                            self.state = State::AutomationFlowEditor;
                            self.update_action_list();
                            self.flow_editor.new_flow();
                        }
                        get_started::GetStartedMessage::OpenAction => {
                            return Command::perform(
                                rfd::AsyncFileDialog::new()
                                    .add_filter("TestAngel Actions", &["taaction"])
                                    .set_title("Open Action")
                                    .set_directory(
                                        env::var("TA_ACTION_DIR").unwrap_or("./actions".to_owned()),
                                    )
                                    .pick_file(),
                                |ret| AppMessage::OpenAction(ret.map(|f| f.path().to_path_buf())),
                            );
                        }
                        get_started::GetStartedMessage::OpenFlow => {
                            return Command::perform(
                                rfd::AsyncFileDialog::new()
                                    .add_filter("TestAngel Flows", &["taflow"])
                                    .set_title("Open Flow")
                                    .set_directory(
                                        env::var("TA_FLOW_DIR").unwrap_or(".".to_owned()),
                                    )
                                    .pick_file(),
                                |ret| AppMessage::OpenFlow(ret.map(|f| f.path().to_path_buf())),
                            );
                        }
                    }
                }
                if let Some(cmd) = cmd {
                    return cmd;
                }
            }
            AppMessage::OpenAction(maybe_file) => {
                if let Some(file) = maybe_file {
                    match self.action_editor.open_action(file) {
                        Ok(_) => self.state = State::ActionEditor,
                        Err(e) => {
                            return Command::perform(
                                rfd::AsyncMessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_title("Failed to open action")
                                    .set_description(&format!("{e}"))
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show(),
                                |_| AppMessage::NoOp,
                            )
                        }
                    }
                }
            }
            AppMessage::OpenFlow(maybe_file) => {
                if let Some(file) = maybe_file {
                    self.update_action_list();
                    match self.flow_editor.open_flow(file) {
                        Ok(changed) => {
                            self.state = State::AutomationFlowEditor;
                            return Command::perform(rfd::AsyncMessageDialog::new()
                                .set_level(rfd::MessageLevel::Warning)
                                .set_title("Action has changed")
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_description(&format!(
                                    "The parameters in steps {} have changed so it has been reset.",
                                    changed.iter().map(|step| step.to_string()).collect::<Vec<_>>().join(",")
                                ))
                                .show(), |_| AppMessage::NoOp);
                        }
                        Err(e) => {
                            return Command::perform(
                                rfd::AsyncMessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_title("Failed to open flow")
                                    .set_description(&format!("{e}"))
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show(),
                                |_| AppMessage::NoOp,
                            )
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Render content
        let content: Element<'_, AppMessage> = match self.state {
            State::GetStarted => self.get_started.view().map(AppMessage::GetStarted),
            State::ActionEditor => self.action_editor.view().map(AppMessage::ActionEditor),
            State::AutomationFlowEditor => self.flow_editor.view().map(AppMessage::FlowEditor),
            State::AutomationFlowRunning => self.flow_running.view().map(AppMessage::FlowRunning),
        };

        content
    }
}

trait UiComponent {
    type Message: Debug + Send;
    type MessageOut: Debug + Send;

    fn title(&self) -> Option<&str>;

    /// Handle a message.
    fn update(
        &mut self,
        message: Self::Message,
    ) -> (Option<Self::MessageOut>, Option<Command<AppMessage>>);

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Render the central panel UI.
    fn view(&self) -> Element<'_, Self::Message>;
}
