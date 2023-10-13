use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration, ActionParameterSource};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

/// The data object to hold the data for initialising an [`ActionComponent`].
pub struct ActionComponentInitialiser {
    pub possible_outputs: Vec<(String, ParameterKind, ActionParameterSource)>,
    pub config: ActionConfiguration,
    pub action: Action,
}

#[derive(Debug)]
pub struct ActionComponent {
    step: DynamicIndex,
    config: ActionConfiguration,
    action: Action,
    visible: bool,

    possible_outputs: Vec<(String, ParameterKind, ActionParameterSource)>,
    variable_rows: FactoryVecDeque<VariableRow>,
}

#[derive(Debug)]
pub enum ActionComponentInput {
    SetVisible(bool),
    NewSourceFor(usize, ActionParameterSource),
    NewValueFor(usize, ParameterValue),
}

#[derive(Debug)]
pub enum ActionComponentOutput {
    /// (Base index, Offset)
    Cut(DynamicIndex),
    Paste(usize, ActionConfiguration),
    Remove(DynamicIndex),
    ConfigUpdate(DynamicIndex, ActionConfiguration),
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
                set_title: &t!("flows.action-component.label", step = self.step.current_index() + 1, name = self.action.friendly_name),
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
        index: &Self::Index,
        sender: relm4::FactorySender<Self>,
    ) -> Self {
        let ActionComponentInitialiser {
            possible_outputs,
            action,
            config,
        } = init;

        Self {
            step: index.clone(),
            possible_outputs,
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
                let possible_sources = self
                    .possible_outputs
                    .iter()
                    .filter(|(_, o_kind, _)| o_kind == kind)
                    .map(|(a, _, c)| (a.clone(), c.clone()))
                    .collect();

                variable_rows.push_back((
                    idx,
                    name.clone(),
                    *kind,
                    self.config.parameter_sources[&idx].clone(),
                    self.config.parameter_values[&idx].clone(),
                    [
                        vec![(
                            t!("flows.action-component.source-literal"),
                            ActionParameterSource::Literal,
                        )],
                        possible_sources,
                    ]
                    .concat(),
                ));
            }
        }

        let row = self.variable_rows.widget();
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: relm4::FactorySender<Self>) {
        match message {
            ActionComponentInput::SetVisible(to) => self.visible = to,
            ActionComponentInput::NewSourceFor(idx, source) => {
                self.config.parameter_sources.insert(idx, source);
                sender.output(ActionComponentOutput::ConfigUpdate(
                    self.step.clone(),
                    self.config.clone(),
                ));
            }
            ActionComponentInput::NewValueFor(idx, source) => {
                self.config.parameter_values.insert(idx, source);
                sender.output(ActionComponentOutput::ConfigUpdate(
                    self.step.clone(),
                    self.config.clone(),
                ));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        match output {
            ActionComponentOutput::Remove(idx) => Some(super::FlowInputs::RemoveStep(idx)),
            ActionComponentOutput::Cut(idx) => Some(super::FlowInputs::CutStep(idx)),
            ActionComponentOutput::Paste(idx, step) => {
                Some(super::FlowInputs::PasteStep(idx, step))
            }
            ActionComponentOutput::ConfigUpdate(step, config) => {
                Some(super::FlowInputs::ConfigUpdate(step, config))
            }
        }
    }
}

#[derive(Debug)]
struct VariableRow {
    idx: usize,
    name: String,
    kind: ParameterKind,
    source: ActionParameterSource,
    value: ParameterValue,

    potential_sources_raw: Vec<(String, ActionParameterSource)>,
    potential_sources: FactoryVecDeque<SourceSearchResult>,
}

impl VariableRow {
    fn get_nice_name_for(&self, source: &ActionParameterSource) -> String {
        for (name, src) in &self.potential_sources_raw {
            if *src == *source {
                return name.clone();
            }
        }

        source.to_string()
    }
}

#[derive(Debug)]
enum VariableRowInput {
    SourceSelected(ActionParameterSource),
    ChangeValue(ParameterValue),
}

#[derive(Debug)]
enum VariableRowOutput {
    NewSourceFor(usize, ActionParameterSource),
    NewValueFor(usize, ParameterValue),
}

#[relm4::factory]
impl FactoryComponent for VariableRow {
    type Init = (
        usize,
        String,
        ParameterKind,
        ActionParameterSource,
        ParameterValue,
        Vec<(String, ActionParameterSource)>,
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
                    source = &self.get_nice_name_for(&self.source),
                )
            },

            add_suffix = &gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::EDIT,
                set_tooltip_text: Some(&t!("flows.action-component.edit-param")),
                set_css_classes: &["flat"],
                set_direction: gtk::ArrowType::Left,

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_min_content_height: 150,

                        gtk::Box {
                            set_spacing: 5,
                            set_orientation: gtk::Orientation::Vertical,

                            adw::Bin {
                                #[watch]
                                set_visible: self.source == ActionParameterSource::Literal,

                                #[transition = "None"]
                                match self.kind {
                                    ParameterKind::String => {
                                        gtk::Entry {
                                            set_text: &self.value.value_string(),
                                            set_placeholder_text: Some(&t!("value")),

                                            connect_changed[sender] => move |btn| {
                                                sender.input(VariableRowInput::ChangeValue(ParameterValue::String(btn.text().to_string())));
                                            },
                                        }
                                    }
                                    ParameterKind::Integer => {
                                        gtk::SpinButton {
                                            set_digits: 0,
                                            #[watch]
                                            set_value: self.value.value_i32() as f64,
                                            set_increments: (1., 10.),
                                            set_numeric: true,

                                            connect_changed[sender] => move |btn| {
                                                if let Ok(val) = btn.text().parse::<i32>() {
                                                    sender.input(VariableRowInput::ChangeValue(ParameterValue::Integer(val)));
                                                }
                                            },
                                        }
                                    }
                                    ParameterKind::Decimal => {
                                        gtk::SpinButton {
                                            set_digits: 2,
                                            #[watch]
                                            set_value: self.value.value_f32() as f64,
                                            set_increments: (0.1, 1.),
                                            set_numeric: true,

                                            connect_changed[sender] => move |btn| {
                                                if let Ok(val) = btn.text().parse::<f32>() {
                                                    sender.input(VariableRowInput::ChangeValue(ParameterValue::Decimal(val)));
                                                }
                                            },
                                        }
                                    }
                                    ParameterKind::Boolean => {
                                        gtk::CheckButton {
                                            set_label: Some(&t!("value")),
                                            #[watch]
                                            set_active: self.value.value_bool(),

                                            connect_toggled[sender] => move |btn| {
                                                sender.input(VariableRowInput::ChangeValue(ParameterValue::Boolean(btn.is_active())));
                                            },
                                        }
                                    }
                                },
                            },

                            #[local_ref]
                            potential_sources -> gtk::Box {
                                set_spacing: 5,
                                set_orientation: gtk::Orientation::Vertical,
                            },
                        },
                    }
                },
            },
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        sender: relm4::FactorySender<Self>,
    ) -> Self {
        let mut potential_sources =
            FactoryVecDeque::new(gtk::Box::default(), sender.input_sender());
        {
            // populate sources
            let mut potential_sources = potential_sources.guard();
            for (label, source) in init.5.clone() {
                potential_sources.push_back((label, source));
            }
        }

        Self {
            idx: init.0,
            name: init.1,
            kind: init.2,
            source: init.3,
            value: init.4,
            potential_sources_raw: init.5,
            potential_sources,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let potential_sources = self.potential_sources.widget();
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            VariableRowInput::SourceSelected(new_source) => {
                self.source = new_source.clone();
                sender.output(VariableRowOutput::NewSourceFor(self.idx, new_source));
            }
            VariableRowInput::ChangeValue(new_value) => {
                self.value = new_value.clone();
                sender.output(VariableRowOutput::NewValueFor(self.idx, new_value));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        match output {
            VariableRowOutput::NewSourceFor(idx, source) => {
                Some(ActionComponentInput::NewSourceFor(idx, source))
            }
            VariableRowOutput::NewValueFor(idx, source) => {
                Some(ActionComponentInput::NewValueFor(idx, source))
            }
        }
    }
}

#[derive(Debug)]
struct SourceSearchResult {
    label: String,
    source: ActionParameterSource,
}

#[derive(Debug)]
enum SourceSearchResultInput {
    Select,
}

#[relm4::factory]
impl FactoryComponent for SourceSearchResult {
    type Init = (String, ActionParameterSource);
    type Input = SourceSearchResultInput;
    type Output = ActionParameterSource;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type ParentInput = VariableRowInput;

    view! {
        root = gtk::Button::builder().css_classes(["flat"]).build() {
            set_label: &self.label,

            connect_clicked => SourceSearchResultInput::Select,
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            label: init.0,
            source: init.1,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            SourceSearchResultInput::Select => sender.output(self.source.clone()),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(VariableRowInput::SourceSelected(output))
    }
}
