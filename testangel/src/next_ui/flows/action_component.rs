use adw::prelude::*;
use gtk::prelude::*;
use relm4::{
    adw, gtk,
    prelude::{DynamicIndex, FactoryComponent},
    ComponentParts, RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration};

/// The data object to hold the data for initialising an [`ActionComponent`].
pub struct ActionComponentInitialiser {
    pub step: usize,
    pub config: ActionConfiguration,
    pub action: Action,
}

#[derive(Debug)]
pub struct ActionComponent {
    step: usize,
    config: ActionConfiguration,
    action: Action,
}

#[derive(Debug)]
pub enum ActionComponentOutput {
    MoveTo(DynamicIndex),
    Remove(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for ActionComponent {
    type Init = ActionComponentInitialiser;
    type Input = ();
    type Output = ActionComponentOutput;
    type CommandOutput = ();
    type ParentInput = super::FlowInputs;
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ActionRow {
            #[watch]
            set_title: &t!("flows.action-component.label", step = self.step + 1, name = self.action.friendly_name),
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        let ActionComponentInitialiser {
            step,
            action,
            config,
        } = init;
        Self {
            step,
            config,
            action,
        }
    }
}
