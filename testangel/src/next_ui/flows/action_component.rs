use gtk::prelude::*;
use relm4::{gtk, ComponentParts, RelmWidgetExt, SimpleComponent};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration};

/// The data object to hold the data for initialising an [`ActionComponent`].
pub struct ActionComponentInitialiser {
    step: usize,
    config: ActionConfiguration,
    action: Action,
}

#[derive(Debug)]
pub struct ActionComponentModel {
    step: usize,
    config: ActionConfiguration,
    action: Action,
}

impl ActionComponentModel {
    /// Update the step number that this action consists of.
    pub fn set_step(&mut self, step: usize) {
        self.step = step;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ActionComponentModel {
    type Init = ActionComponentInitialiser;
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Frame {
            #[watch]
            set_label: Some(&t!("flows.action-component.label", step = model.step, label = model.action.friendly_name)),

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Label {
                    set_label: &model.action.description,
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let ActionComponentInitialiser {
            step,
            action,
            config,
        } = init;
        let model = ActionComponentModel {
            step,
            config,
            action,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
