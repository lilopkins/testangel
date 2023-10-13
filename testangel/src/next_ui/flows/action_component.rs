use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    RelmWidgetExt,
};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration, ActionParameterSource};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

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
    visible: bool,

    variable_rows: FactoryVecDeque<VariableRow>,
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
    type ParentWidget = gtk::Box;

    view! {
        root = adw::Bin {
            set_margin_all: 5,

            #[local_ref]
            row -> adw::PreferencesGroup {
                #[watch]
                set_title: &t!("flows.action-component.label", step = self.step + 1, name = self.action.friendly_name),
                set_description: Some(&self.action.description),
                #[watch]
                set_visible: self.visible,

                #[wrap(Some)]
                set_header_suffix = &gtk::Box {
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
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        sender: relm4::FactorySender<Self>,
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
            variable_rows: FactoryVecDeque::new(
                adw::PreferencesGroup::default(),
                sender.input_sender(),
            ),
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

        {
            // initialise rows
            let mut variable_rows = self.variable_rows.guard();
            for (idx, (name, kind)) in self.action.parameters.iter().enumerate() {
                variable_rows.push_back((
                    idx,
                    name.clone(),
                    *kind,
                    self.config.parameter_sources[&idx].clone(),
                    self.config.parameter_values[&idx].clone(),
                ));
            }
        }

        let row = self.variable_rows.widget();
        let widgets = view_output!();

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
            ActionComponentOutput::Paste(idx, step) => {
                Some(super::FlowInputs::PasteStep(idx, step))
            }
        }
    }
}

#[derive(Debug)]
struct VariableRow {
    name: String,
    kind: ParameterKind,
    source: ActionParameterSource,
    value: ParameterValue,
}

#[derive(Debug)]
enum VariableRowInput {}

#[derive(Debug)]
enum VariableRowOutput {}

#[relm4::factory]
impl FactoryComponent for VariableRow {
    type Init = (
        usize,
        String,
        ParameterKind,
        ActionParameterSource,
        ParameterValue,
    );
    type Input = VariableRowInput;
    type Output = VariableRowOutput;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;
    type ParentInput = ActionComponentInput;

    view! {
        adw::ActionRow {
            set_title: &self.name,
            #[watch]
            set_subtitle: &if self.source == ActionParameterSource::Literal {
                t!(
                    "flows.action-component.subtitle-with-value",
                    kind = self.kind,
                    source = self.source,
                    value = self.value,
                )
            } else {
                t!(
                    "flows.action-component.subtitle",
                    kind = self.kind,
                    source = self.source,
                )
            },

            add_suffix = &gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::EDIT,
                set_tooltip_text: Some(&t!("flows.action-component.edit-param")),
                set_css_classes: &["flat"],
                set_direction: gtk::ArrowType::Left,
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self {
            name: init.1,
            kind: init.2,
            source: init.3,
            value: init.4,
        }
    }
}
