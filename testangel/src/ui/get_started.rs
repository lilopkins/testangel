use iced::widget::{column, row, Button, Container, Rule, Space, Text};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum GetStartedMessage {
    NewFlow,
    OpenFlow,
    NewAction,
    OpenAction,
}

#[derive(Default)]
pub struct GetStarted;

impl UiComponent for GetStarted {
    type Message = GetStartedMessage;
    type MessageOut = GetStartedMessage;

    fn title(&self) -> Option<&str> {
        None
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
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

                Text::new("\nDesign Actions").size(24),
                Text::new("Actions are environment specific, granular but high level pieces of automation. They bring together generic low level instructions."),
                row![
                    Button::new("Create new action")
                        .on_press(GetStartedMessage::NewAction),
                    Button::new("Open existing action")
                        .on_press(GetStartedMessage::OpenAction),
                ].spacing(8),

                Space::with_height(64),
                Text::new(format!("TestAngel v{}", env!("CARGO_PKG_VERSION"))).size(10),
            ].spacing(4)
        )
        .padding([32, 32, 32, 32])
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        Some(message)
    }
}
