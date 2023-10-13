use gtk::prelude::*;
use adw::prelude::*;
use relm4::{
    adw, gtk,
    prelude::{DynamicIndex, FactoryComponent}, RelmWidgetExt,
};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration, ActionParameterSource};

/// The data object to hold the data for initialising an [`ActionComponent`].
pub struct ActionComponentInitialiser {
    pub step: usize,
    pub config: ActionConfiguration,
    pub action: Action,
}

#[derive(Clone, Debug)]
pub struct ActionComponent {
    step: usize,
    config: ActionConfiguration,
    action: Action,
    visible: bool,
}

#[derive(Debug)]
pub enum ActionComponentInput {
    SetVisible(bool),
}

#[derive(Debug)]
pub enum ActionComponentOutput {
    /// (Base index, Offset)
    Cut(DynamicIndex),
    Paste(usize, ActionConfiguration),
    Remove(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for ActionComponent {
    type Init = ActionComponentInitialiser;
    type Input = ActionComponentInput;
    type Output = ActionComponentOutput;
    type CommandOutput = ();
    type ParentInput = super::FlowInputs;
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ExpanderRow {
            #[watch]
            set_title: &t!("flows.action-component.label", step = self.step + 1, name = self.action.friendly_name),
            set_subtitle: &self.action.description,
            set_icon_name: Some(relm4_icons::icon_name::SIZE_VERTICALLY),
            #[watch]
            set_visible: self.visible,

            add_controller = gtk::DragSource {
                set_actions: gtk::gdk::DragAction::MOVE,

                connect_prepare[index] => move |_src, _x, _y| {
                    Some(relm4::gtk::gdk::ContentProvider::for_value(&gtk::glib::Value::from(index.clone().current_index() as u64)))
                },

                connect_drag_begin[sender] => move |_src, _drag| {
                    sender.input(ActionComponentInput::SetVisible(false))
                },

                connect_drag_end[sender] => move |_src, _drag, delete| {
                    if !delete {
                        sender.input(ActionComponentInput::SetVisible(true))
                    }
                },
            },
            add_controller = gtk::DropTarget {

            },

            add_row = &adw::ActionRow {
                set_title: &t!("flows.action-component.actions"),
                add_suffix = &gtk::Box {
                    set_spacing: 5,

                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::UP,
                        set_tooltip: &t!("flows.move-up"),

                        connect_clicked[sender, index, config] => move |_| {
                            if index.clone().current_index() != 0 {
                                sender.output(ActionComponentOutput::Cut(index.clone()));
                                sender.output(ActionComponentOutput::Paste((index.clone().current_index() - 1).max(0), config.clone()));
                            }
                        },
                    },
                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::DOWN,
                        set_tooltip: &t!("flows.move-down"),

                        connect_clicked[sender, index, config] => move |_| {
                            sender.output(ActionComponentOutput::Cut(index.clone()));
                            sender.output(ActionComponentOutput::Paste(index.clone().current_index() + 1, config.clone()));
                        },
                    },
                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::X_CIRCULAR,
                        set_tooltip: &t!("flows.action-component.delete"),

                        connect_clicked[sender, index] => move |_| {
                            sender.output(ActionComponentOutput::Remove(index.clone()));
                        },
                    },
                },
            },
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
            visible: true,
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: relm4::FactorySender<Self>,
    ) -> Self::Widgets {
        let config = self.config.clone();
        let widgets = view_output!();
        for (idx, (name, kind)) in self.action.parameters.iter().enumerate() {
            let src = &self.config.parameter_sources[&idx];

            let row = adw::ActionRow::builder()
                .title(name)
                .subtitle(if *src == ActionParameterSource::Literal {
                    t!(
                        "flows.action-component.subtitle-with-value",
                        kind = kind,
                        source = src,
                        value = self.config.parameter_values[&idx],
                    )
                } else {
                    t!(
                        "flows.action-component.subtitle",
                        kind = kind,
                        source = src
                    )
                })
                .build();
            let edit_btn = gtk::Button::builder()
                .icon_name(relm4_icons::icon_name::EDIT)
                .tooltip_text(t!("flows.action-component.edit-param"))
                .css_classes(["flat"])
                .build();
            row.add_suffix(&edit_btn);
            edit_btn.connect_clicked(|_| todo!("show source and literal edit dialog here"));
            widgets.root.add_row(&row);
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::FactorySender<Self>) {
        match message {
            ActionComponentInput::SetVisible(to) => self.visible = to,
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        match output {
            ActionComponentOutput::Remove(idx) => Some(super::FlowInputs::RemoveStep(idx)),
            ActionComponentOutput::Cut(idx) => Some(super::FlowInputs::CutStep(idx)),
            ActionComponentOutput::Paste(idx, step) => Some(super::FlowInputs::PasteStep(idx, step)),
        }
    }
}
