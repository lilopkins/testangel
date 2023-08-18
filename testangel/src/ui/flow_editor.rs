use super::UiComponent;

#[derive(Clone, Debug)]
pub enum FlowEditorMessage {}

#[derive(Default)]
pub struct FlowEditor {}

impl FlowEditor {
    pub(crate) fn new_flow(&self) {
        todo!()
    }

    pub(crate) fn open_flow(&self, file: std::path::PathBuf) {
        todo!()
    }
}

impl UiComponent for FlowEditor {
    type Message = FlowEditorMessage;
    type MessageOut = ();

    fn title(&self) -> Option<&str> {
        None
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        todo!()
    }

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        todo!()
    }
}
