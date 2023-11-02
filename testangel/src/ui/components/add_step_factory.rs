use std::fmt::Debug;
use std::marker::PhantomData;

use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

pub trait AddStepTrait {
    fn add_step(value: String) -> Self;
}

pub struct AddStepInit {
    pub label: String,
    pub value: String,
}

#[derive(Debug)]
pub struct AddStepResult<PI: AddStepTrait + Debug + 'static> {
    _pi: PhantomData<PI>,
    label: String,
    value: String,
}

impl<PI: AddStepTrait + Debug + 'static> AddStepResult<PI> {
    /// Get the instruction ID this result references
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[relm4::factory(pub)]
impl<PI: AddStepTrait + Debug + 'static> FactoryComponent for AddStepResult<PI> {
    type Init = AddStepInit;
    type Input = ();
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type ParentInput = PI;

    view! {
        root = gtk::Button::builder().css_classes(["flat"]).build() {
            set_label: &self.label,

            connect_clicked[sender, id] => move |_| {
                sender.output(id.clone())
            }
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            _pi: PhantomData,
            label: init.label,
            value: init.value,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.value.clone();
        let widgets = view_output!();
        widgets
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(PI::add_step(output))
    }
}
