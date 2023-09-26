use std::env;

use iced::widget::{column, row, Button, Container, Rule, Space, Text};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum GetStartedMessage {
    NewFlow,
    OpenFlow,
    NewAction,
    OpenAction,
}

pub struct GetStarted {
    is_latest: bool,
    hide_action_editor: bool,
    local_help_contact: Option<String>,
}
impl GetStarted {
    pub fn set_is_latest(&mut self, is_latest: bool) {
        self.is_latest = is_latest;
    }
}

impl Default for GetStarted {
    fn default() -> Self {
        Self {
            is_latest: true,
            hide_action_editor: !env::var("TA_HIDE_ACTION_EDITOR")
                .unwrap_or("no".to_string())
                .eq_ignore_ascii_case("no"),
            local_help_contact: env::var("TA_LOCAL_SUPPORT_CONTACT")
                .map(Some)
                .unwrap_or(None),
        }
    }
}

impl UiComponent for GetStarted {
    type Message = GetStartedMessage;
    type MessageOut = GetStartedMessage;

    fn title(&self) -> Option<&str> {
        None
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let ae: iced::Element<'_, Self::Message> = if self.hide_action_editor {
            Space::new(0, 0).into()
        } else {
            column![
                Text::new("\nDesign Actions").size(24),
                Text::new("Actions are environment specific, granular but high level pieces of automation. They bring together generic low level instructions."),
                row![
                    Button::new("Create new action")
                        .on_press(GetStartedMessage::NewAction),
                    Button::new("Open existing action")
                        .on_press(GetStartedMessage::OpenAction),
                ].spacing(8),
            ].spacing(4).into()
        };

        let help: iced::Element<'_, Self::Message> =
            Text::new(if let Some(contact) = &self.local_help_contact {
                format!("For help, please contact: {contact}")
            } else {
                String::from("Repository: https://github.com/lilopkins/testangel")
            })
            .into();

        Container::new(
            column![
                Text::new("Get Started with TestAngel").size(32),
                Rule::horizontal(2),

                Text::new("\nDesign Flows").size(24),
                Text::new("Flows allow you to automate project-specific tasks and tests. They bring together high level actions into a complete flow."),
                row![
                    Button::new("Create new flow")
                        .on_press(GetStartedMessage::NewFlow),
                    Button::new("Open existing flow")
                        .on_press(GetStartedMessage::OpenFlow),
                ].spacing(8),

                ae,

                Space::with_height(64),

                help,
                Text::new(format!("TestAngel v{}{}", env!("CARGO_PKG_VERSION"), if self.is_latest { "" } else { " - New version available!" })).size(10),
            ].spacing(4)
        )
        .padding([32, 32, 32, 32])
        .into()
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> (
        Option<Self::MessageOut>,
        Option<iced::Command<super::AppMessage>>,
    ) {
        (Some(message), None)
    }
}
