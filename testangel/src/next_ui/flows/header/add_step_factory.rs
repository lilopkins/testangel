use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;
use testangel::types::Action;

#[derive(Debug)]
pub struct StepSearchResult {
    name: String,
    action_id: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for StepSearchResult {
    type Init = Action;
    type Input = ();
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type ParentInput = super::FlowsHeaderInput;

    view! {
        root = gtk::Button::builder().css_classes(["flat"]).build() {
            set_label: &self.name,

            connect_clicked[sender, id] => move |_| {
                sender.output(id.clone())
            }
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            name: format!("{}: {}", init.group, init.friendly_name),
            action_id: init.id,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.action_id.clone();
        let widgets = view_output!();
        widgets
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(super::FlowsHeaderInput::AddStep(output))
    }
}
