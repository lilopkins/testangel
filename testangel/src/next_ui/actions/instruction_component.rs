use std::{collections::HashMap, ffi};

use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    RelmWidgetExt,
};
use testangel::types::{InstructionConfiguration, InstructionParameterSource};
use testangel_ipc::prelude::{Instruction, ParameterKind, ParameterValue};

use crate::next_ui::{
    components::variable_row::{
        ParameterSourceTrait, VariableRow, VariableRowInit, VariableRowParentInput,
    },
    lang,
};

/// The data object to hold the data for initialising an [`ActionComponent`].
pub struct InstructionComponentInitialiser {
    pub possible_outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    pub config: InstructionConfiguration,
    pub instruction: Instruction,
}

#[derive(Debug)]
pub struct InstructionComponent {
    step: DynamicIndex,
    config: InstructionConfiguration,
    instruction: Instruction,
    visible: bool,

    possible_outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    possible_run_conditions: Vec<(String, InstructionParameterSource)>,
    run_condition_index: u32,
    variable_rows:
        FactoryVecDeque<VariableRow<InstructionParameterSource, String, InstructionComponentInput>>,

    /// True when a drag-and-drop operation is proposed to add a component above this one
    drop_proposed_above: bool,
    /// True when a drag-and-drop operation is proposed to add a component below this one
    drop_proposed_below: bool,
}

#[derive(Debug)]
pub enum InstructionComponentInput {
    SetComment(String),
    SetVisible(bool),
    NewSourceFor(String, InstructionParameterSource),
    NewValueFor(String, ParameterValue),
    ProposedDrop { above: bool, below: bool },
    ChangeRunCondition(u32),
}

impl VariableRowParentInput<String, InstructionParameterSource> for InstructionComponentInput {
    fn new_source_for(idx: String, new_source: InstructionParameterSource) -> Self {
        Self::NewSourceFor(idx, new_source)
    }

    fn new_value_for(idx: String, new_value: ParameterValue) -> Self {
        Self::NewValueFor(idx, new_value)
    }
}

impl ParameterSourceTrait for InstructionParameterSource {
    fn literal() -> Self {
        Self::Literal
    }
}

#[derive(Debug)]
pub enum InstructionComponentOutput {
    /// (Base index, Offset)
    Cut(DynamicIndex),
    Paste(usize, InstructionConfiguration),
    Remove(DynamicIndex),
    ConfigUpdate(DynamicIndex, InstructionConfiguration),
    /// (from, to, offset)
    MoveStep(DynamicIndex, DynamicIndex, isize),
    ChangeRunCondition(DynamicIndex, InstructionParameterSource),
    SetComment(DynamicIndex, String),
}

#[relm4::factory(pub)]
impl FactoryComponent for InstructionComponent {
    type Init = InstructionComponentInitialiser;
    type Input = InstructionComponentInput;
    type Output = InstructionComponentOutput;
    type CommandOutput = ();
    type ParentInput = super::ActionInputs;
    type ParentWidget = gtk::Box;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Label {
                set_label: &lang::lookup("drag-drop-here"),
                #[watch]
                set_visible: self.drop_proposed_above,
            },

            #[local_ref]
            row -> adw::PreferencesGroup {
                #[watch]
                set_title: &lang::lookup_with_args(
                    "action-step-label",
                    {
                        let mut map = HashMap::new();
                        map.insert("step", (self.step.current_index() + 1).into());
                        map.insert("name", self.instruction.friendly_name().clone().into());
                        map
                    }
                ),
                #[watch]
                set_description: Some(&format!("{}\n{}", self.instruction.description(), self.config.comment)),
                #[watch]
                set_visible: self.visible,

                #[wrap(Some)]
                set_header_suffix = &gtk::Box {
                    set_spacing: 5,

                    gtk::DropDown {
                        set_model: Some(
                            &gtk::StringList::new(self.possible_run_conditions.iter()
                                .map(|(label, _)| label.as_str())
                                .collect::<Vec<_>>()
                                .as_slice())),
                        set_selected: self.run_condition_index,

                        connect_selected_notify[sender] => move |dropdown| {
                            let idx = dropdown.selected();
                            sender.input(InstructionComponentInput::ChangeRunCondition(idx));
                        },
                    },
                    gtk::MenuButton::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::TAG,
                        set_tooltip: &lang::lookup("action-step-set-comment"),

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            gtk::Entry {
                                set_text: &self.config.comment,

                                connect_changed[sender, index] => move |entry| {
                                    sender.input(InstructionComponentInput::SetComment(entry.text().to_string()));
                                    sender.output(InstructionComponentOutput::SetComment(index.clone(), entry.text().to_string()));
                                },
                            }
                        },
                    },
                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::UP,
                        set_tooltip: &lang::lookup("move-up"),

                        connect_clicked[sender, index, config] => move |_| {
                            if index.clone().current_index() != 0 {
                                sender.output(InstructionComponentOutput::Cut(index.clone()));
                                sender.output(InstructionComponentOutput::Paste((index.clone().current_index() - 1).max(0), config.clone()));
                            }
                        },
                    },
                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::DOWN,
                        set_tooltip: &lang::lookup("move-down"),

                        connect_clicked[sender, index, config] => move |_| {
                            sender.output(InstructionComponentOutput::Cut(index.clone()));
                            sender.output(InstructionComponentOutput::Paste(index.clone().current_index() + 1, config.clone()));
                        },
                    },
                    gtk::Button::builder().css_classes(["flat"]).build() {
                        set_icon_name: relm4_icons::icon_name::X_CIRCULAR,
                        set_tooltip: &lang::lookup("delete-step"),

                        connect_clicked[sender, index] => move |_| {
                            sender.output(InstructionComponentOutput::Remove(index.clone()));
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
                        sender.input(InstructionComponentInput::SetVisible(false))
                    },

                    connect_drag_end[sender] => move |_src, _drag, delete| {
                        if !delete {
                            sender.input(InstructionComponentInput::SetVisible(true))
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
                            sender.output(InstructionComponentOutput::MoveStep(*from, to, offset));
                            sender.input(InstructionComponentInput::ProposedDrop { above: false, below: false, });
                            return true;
                        }
                        false
                    },

                    connect_enter[sender] => move |drop, _x, y| {
                        let half = drop.widget().height() as f64 / 2.0;
                        if y < half {
                            // top half
                            sender.input(InstructionComponentInput::ProposedDrop { above: true, below: false, });
                        } else {
                            // bottom half
                            sender.input(InstructionComponentInput::ProposedDrop { above: false, below: true, });
                        }
                        gtk::gdk::DragAction::MOVE
                    },

                    connect_motion[sender] => move |drop, _x, y| {
                        let half = drop.widget().height() as f64 / 2.0;
                        if y < half {
                            // top half
                            sender.input(InstructionComponentInput::ProposedDrop { above: true, below: false, });
                        } else {
                            // bottom half
                            sender.input(InstructionComponentInput::ProposedDrop { above: false, below: true, });
                        }
                        gtk::gdk::DragAction::MOVE
                    },

                    connect_leave => InstructionComponentInput::ProposedDrop { above: false, below: false, },
                },
            },

            gtk::Label {
                set_label: &lang::lookup("drag-drop-here"),
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
        let InstructionComponentInitialiser {
            possible_outputs,
            instruction,
            config,
        } = init;

        let possible_run_conditions = [
            vec![(
                lang::lookup("action-condition-run-always"),
                InstructionParameterSource::Literal,
            )],
            possible_outputs
                .iter()
                .filter(|(_, kind, _)| *kind == ParameterKind::Boolean)
                .map(|(label, _, src)| {
                    (
                        lang::lookup_with_args("action-condition-run-condition", {
                            let mut map = HashMap::new();
                            map.insert("cond", label.clone().into());
                            map
                        }),
                        src.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        ]
        .concat();
        let sender_c = sender.clone();
        let run_condition_index = possible_run_conditions
            .iter()
            .enumerate()
            .find(|(_, (_, src))| *src == config.run_if)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| {
                // fix a potentially very broken situation
                sender_c.output(InstructionComponentOutput::ChangeRunCondition(
                    index.clone(),
                    InstructionParameterSource::Literal,
                ));
                log::warn!(
                    "Fixed bad pointing run condition! This fix should never have been called!"
                );
                0
            }) as u32;

        Self {
            step: index.clone(),
            possible_outputs,
            possible_run_conditions,
            run_condition_index,
            config,
            instruction,
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
            for (id, (name, kind)) in self.instruction.parameters().iter() {
                let possible_sources = self
                    .possible_outputs
                    .iter()
                    .filter(|(_, o_kind, _)| o_kind == kind)
                    .map(|(a, _, c)| (a.clone(), c.clone()))
                    .collect();

                variable_rows.push_back(VariableRowInit {
                    index: id.clone(),
                    name: name.clone(),
                    kind: *kind,
                    current_source: self.config.parameter_sources[id].clone(),
                    current_value: self.config.parameter_values[id].clone(),
                    potential_sources: [
                        vec![(
                            lang::lookup("source-literal"),
                            InstructionParameterSource::Literal,
                        )],
                        possible_sources,
                    ]
                    .concat(),
                });
            }
        }

        let row = self.variable_rows.widget();
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: relm4::FactorySender<Self>) {
        match message {
            InstructionComponentInput::SetVisible(to) => self.visible = to,
            InstructionComponentInput::SetComment(comment) => {
                self.config.comment = comment;
            }
            InstructionComponentInput::NewSourceFor(idx, source) => {
                self.config.parameter_sources.insert(idx, source);
                sender.output(InstructionComponentOutput::ConfigUpdate(
                    self.step.clone(),
                    self.config.clone(),
                ));
            }
            InstructionComponentInput::NewValueFor(idx, source) => {
                self.config.parameter_values.insert(idx, source);
                sender.output(InstructionComponentOutput::ConfigUpdate(
                    self.step.clone(),
                    self.config.clone(),
                ));
            }
            InstructionComponentInput::ProposedDrop { above, below } => {
                self.drop_proposed_above = above;
                self.drop_proposed_below = below;
            }
            InstructionComponentInput::ChangeRunCondition(idx) => {
                let (_, src) = &self.possible_run_conditions[idx as usize];
                sender.output(InstructionComponentOutput::ChangeRunCondition(
                    self.step.clone(),
                    src.clone(),
                ));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        match output {
            InstructionComponentOutput::Remove(idx) => Some(super::ActionInputs::RemoveStep(idx)),
            InstructionComponentOutput::Cut(idx) => Some(super::ActionInputs::CutStep(idx)),
            InstructionComponentOutput::Paste(idx, step) => {
                Some(super::ActionInputs::PasteStep(idx, step))
            }
            InstructionComponentOutput::ConfigUpdate(step, config) => {
                Some(super::ActionInputs::ConfigUpdate(step, config))
            }
            InstructionComponentOutput::MoveStep(from, to, offset) => {
                Some(super::ActionInputs::MoveStep(from, to, offset))
            }
            InstructionComponentOutput::ChangeRunCondition(step, new_condition) => {
                Some(super::ActionInputs::ChangeRunCondition(step, new_condition))
            }
            InstructionComponentOutput::SetComment(idx, comment) => {
                Some(super::ActionInputs::SetComment(idx, comment))
            }
        }
    }
}
