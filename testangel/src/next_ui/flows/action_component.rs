use std::ffi;

use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    Component, ComponentController, Controller, FactorySender, RelmWidgetExt,
};
use rust_i18n::t;
use testangel::types::{Action, ActionConfiguration, ActionParameterSource};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

use crate::next_ui::components::literal_input::{LiteralInput, LiteralInputOutput};

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

    /// True when a drag-and-drop operation is proposed to add a component above this one
    drop_proposed_above: bool,
    /// True when a drag-and-drop operation is proposed to add a component below this one
    drop_proposed_below: bool,
}

#[derive(Debug)]
pub enum ActionComponentInput {
    SetVisible(bool),
    NewSourceFor(usize, ActionParameterSource),
    NewValueFor(usize, ParameterValue),
    ProposedDrop { above: bool, below: bool },
}

#[derive(Debug)]
pub enum ActionComponentOutput {
    /// (Base index, Offset)
    Cut(DynamicIndex),
    Paste(usize, ActionConfiguration),
    Remove(DynamicIndex),
    ConfigUpdate(DynamicIndex, ActionConfiguration),
    /// (from, to, offset)
    MoveStep(DynamicIndex, DynamicIndex, isize),
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
        root = gtk::Box {
            set_margin_all: 5,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Label {
                set_label: &t!("drag-drop.here"),
                #[watch]
                set_visible: self.drop_proposed_above,
            },

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
                        let p_index = Box::into_raw(Box::new(index.clone())) as *mut ffi::c_void;
                        Some(gtk::gdk::ContentProvider::for_value(&p_index.to_value()))
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
                    set_actions: gtk::gdk::DragAction::MOVE,
                    set_types: &[gtk::glib::Type::POINTER],

                    connect_drop[sender, index] => move |drop, val, _x, y| {
                        log::debug!("type: {}", val.type_());

                        if let Ok(ptr) = val.get::<*mut ffi::c_void>() {
                            let from = unsafe {
                                Box::from_raw(ptr as *mut DynamicIndex)
                            };
                            let to = index.clone();

                            let half = drop.widget().height() as f64 / 2.0;
                            let offset = if y < half {
                                -1
                            } else {
                                1
                            };
                            sender.output(ActionComponentOutput::MoveStep(*from, to, offset));
                            sender.input(ActionComponentInput::ProposedDrop { above: false, below: false, });
                            return true;
                        }
                        false
                    },

                    connect_enter[sender] => move |drop, _x, y| {
                        let half = drop.widget().height() as f64 / 2.0;
                        if y < half {
                            // top half
                            sender.input(ActionComponentInput::ProposedDrop { above: true, below: false, });
                        } else {
                            // bottom half
                            sender.input(ActionComponentInput::ProposedDrop { above: false, below: true, });
                        }
                        gtk::gdk::DragAction::MOVE
                    },

                    connect_motion[sender] => move |drop, _x, y| {
                        let half = drop.widget().height() as f64 / 2.0;
                        if y < half {
                            // top half
                            sender.input(ActionComponentInput::ProposedDrop { above: true, below: false, });
                        } else {
                            // bottom half
                            sender.input(ActionComponentInput::ProposedDrop { above: false, below: true, });
                        }
                        gtk::gdk::DragAction::MOVE
                    },

                    connect_leave => ActionComponentInput::ProposedDrop { above: false, below: false, },
                },
            },

            gtk::Label {
                set_label: &t!("drag-drop.here"),
                #[watch]
                set_visible: self.drop_proposed_below,
            },
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
            drop_proposed_above: false,
            drop_proposed_below: false,
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
            ActionComponentInput::ProposedDrop { above, below } => {
                self.drop_proposed_above = above;
                self.drop_proposed_below = below;
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
            ActionComponentOutput::MoveStep(from, to, offset) => {
                Some(super::FlowInputs::MoveStep(from, to, offset))
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

    literal_input: Controller<LiteralInput>,
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
                                self.literal_input.widget(),
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

        let literal_input =
            LiteralInput::builder()
                .launch(init.4.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    LiteralInputOutput::ValueChanged(new_value) => {
                        VariableRowInput::ChangeValue(new_value)
                    }
                });

        Self {
            idx: init.0,
            name: init.1,
            kind: init.2,
            source: init.3,
            value: init.4,
            literal_input,
            potential_sources_raw: init.5,
            potential_sources,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
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
