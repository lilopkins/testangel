use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    Component, RelmWidgetExt,
};
use testangel::types::{Action, InstructionParameterSource};
use testangel_ipc::prelude::ParameterKind;

use crate::next_ui::lang;

#[derive(Debug)]
pub enum ActionOutputsInput {
    /// Inform the metadata component that the action has changed and as such
    /// it should reload the metadata values
    ChangeAction(Action),
    SetPossibleSources(Vec<(String, ParameterKind, InstructionParameterSource)>),
    /// Create a new parameter
    NewOutput,
    _FromRow(OutputRowOutput),
}

#[derive(Clone, Debug)]
pub enum ActionOutputsOutput {
    /// Set parameters
    SetOutputs(Vec<(String, ParameterKind, InstructionParameterSource)>),
    /// Remove references to the provided index, or reduce any higher than.
    IndexRemoved(usize),
    /// Swap references to the indexes provided
    IndexesSwapped(usize, usize),
}

#[derive(Debug)]
pub struct ActionOutputs {
    possible_sources: Vec<(String, ParameterKind, InstructionParameterSource)>,

    raw_outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    outputs: FactoryVecDeque<OutputRow>,
}

#[relm4::component(pub)]
impl Component for ActionOutputs {
    type Init = ();
    type Input = ActionOutputsInput;
    type Output = ActionOutputsOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Button {
                set_label: &lang::lookup("action-outputs-new"),
                connect_clicked => ActionOutputsInput::NewOutput,
            },

            model.outputs.widget(),
        },
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ActionOutputs {
            raw_outputs: vec![],
            outputs: FactoryVecDeque::new(
                gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(5)
                    .build(),
                sender.input_sender(),
            ),
            possible_sources: vec![],
        };
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ActionOutputsInput::SetPossibleSources(new_sources) => {
                self.possible_sources = new_sources.clone();
                self.outputs
                    .broadcast(OutputRowInput::SetPossibleSources(new_sources));
            }

            ActionOutputsInput::ChangeAction(action) => {
                let mut params = self.outputs.guard();
                params.clear();
                self.raw_outputs.clear();

                // Add each param from action
                for (name, kind, source) in action.outputs {
                    self.raw_outputs.push((name.clone(), kind, source.clone()));
                    params.push_back((Some((name, source)), self.possible_sources.clone()));
                }
            }

            ActionOutputsInput::NewOutput => {
                if self.possible_sources.is_empty() {
                    // Can't add output.
                    return;
                }
                let mut params = self.outputs.guard();
                params.push_back((None, self.possible_sources.clone()));
                self.raw_outputs.push(self.possible_sources[0].clone());
            }

            ActionOutputsInput::_FromRow(actions_output) => match actions_output {
                OutputRowOutput::MoveUp(index) => {
                    let idx = index.current_index();
                    if idx == 0 {
                        return;
                    }
                    let mut params = self.outputs.guard();
                    self.raw_outputs.swap(idx, idx - 1);
                    params.swap(idx, idx - 1);
                    let _ = sender.output(ActionOutputsOutput::IndexesSwapped(idx, idx - 1));
                }
                OutputRowOutput::MoveDown(index) => {
                    let idx = index.current_index();
                    if idx == self.raw_outputs.len() - 1 {
                        return;
                    }
                    let mut params = self.outputs.guard();
                    self.raw_outputs.swap(idx, idx + 1);
                    params.swap(idx, idx + 1);
                    let _ = sender.output(ActionOutputsOutput::IndexesSwapped(idx, idx + 1));
                }
                OutputRowOutput::Delete(index) => {
                    let mut params = self.outputs.guard();
                    self.raw_outputs.remove(index.current_index());
                    params.remove(index.current_index());
                    let _ =
                        sender.output(ActionOutputsOutput::SetOutputs(self.raw_outputs.clone()));

                    let old_idx = index.current_index();
                    let _ = sender.output(ActionOutputsOutput::IndexRemoved(old_idx));
                }
                OutputRowOutput::SetOutputSource(index, new_source) => {
                    self.raw_outputs[index.current_index()].2 = new_source;
                    let _ =
                        sender.output(ActionOutputsOutput::SetOutputs(self.raw_outputs.clone()));
                }
                OutputRowOutput::SetOutputName(index, new_name) => {
                    self.raw_outputs[index.current_index()].0 = new_name;
                    let _ =
                        sender.output(ActionOutputsOutput::SetOutputs(self.raw_outputs.clone()));
                }
            },
        }

        self.update_view(widgets, sender)
    }
}

#[derive(Debug)]
struct OutputRow {
    name: String,
    possible_sources: Vec<(String, ParameterKind, InstructionParameterSource)>,
    src_index: u32,
    inhibit_next_selection: usize,
}

#[derive(Clone, Debug)]
pub enum OutputRowInput {
    SetPossibleSources(Vec<(String, ParameterKind, InstructionParameterSource)>),
    _SetSource(DynamicIndex, u32),
}

#[derive(Clone, Debug)]
pub enum OutputRowOutput {
    SetOutputName(DynamicIndex, String),
    SetOutputSource(DynamicIndex, InstructionParameterSource),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Delete(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for OutputRow {
    type Init = (
        Option<(String, InstructionParameterSource)>,
        Vec<(String, ParameterKind, InstructionParameterSource)>,
    );
    type Input = OutputRowInput;
    type Output = OutputRowOutput;
    type CommandOutput = ();
    type ParentInput = ActionOutputsInput;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            // name
            gtk::Entry {
                set_hexpand: true,
                set_text: &self.name,
                set_placeholder_text: Some(&lang::lookup("action-outputs-name-placeholder")),

                connect_changed[sender, index] => move |entry| {
                    sender.output(OutputRowOutput::SetOutputName(index.clone(), entry.text().to_string()));
                },
            },

            // kind
            #[name = "dropdown"]
            gtk::DropDown {
                set_model: Some(&gtk::StringList::new(self.possible_sources.iter().map(|(label, _, _)| label.as_str()).collect::<Vec<_>>().as_slice())),
                set_selected: self.src_index,

                connect_selected_notify[sender, index] => move |dropdown| {
                    let idx = dropdown.selected();
                    sender.input(OutputRowInput::_SetSource(index.clone(), idx));
                },
            },

            // move up
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::UP,
                set_tooltip: &lang::lookup("move-up"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(OutputRowOutput::MoveUp(index.clone()));
                }
            },
            // move down
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::DOWN,
                set_tooltip: &lang::lookup("move-down"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(OutputRowOutput::MoveDown(index.clone()));
                }
            },
            // delete
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::X_CIRCULAR,
                set_tooltip: &lang::lookup("delete"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(OutputRowOutput::Delete(index.clone()));
                }
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        if let Some((name, p_src)) = init.0 {
            let possible_sources = init.1;
            let mut src_index = gtk::INVALID_LIST_POSITION;
            for (idx, (_, _, src)) in possible_sources.iter().enumerate() {
                if *src == p_src {
                    src_index = idx as u32;
                    break;
                }
            }

            Self {
                name,
                src_index,
                possible_sources,
                inhibit_next_selection: 0,
            }
        } else {
            Self {
                name: String::new(),
                src_index: 0,
                possible_sources: init.1,
                inhibit_next_selection: 0,
            }
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::FactorySender<Self>,
    ) {
        match message {
            OutputRowInput::SetPossibleSources(new_sources) => {
                let selection_index = widgets.dropdown.selected();
                let (_, _, current_source) = &self.possible_sources[selection_index as usize];

                let mut src_index = gtk::INVALID_LIST_POSITION;
                for (idx, (_, _, src)) in new_sources.iter().enumerate() {
                    if *src == *current_source {
                        src_index = idx as u32;
                        break;
                    }
                }

                self.possible_sources = new_sources;
                self.src_index = src_index;
                self.inhibit_next_selection = 2;
                widgets.dropdown.set_model(Some(&gtk::StringList::new(
                    self.possible_sources
                        .iter()
                        .map(|(label, _, _)| label.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                )));
                widgets.dropdown.set_selected(self.src_index);
            }
            OutputRowInput::_SetSource(index, dropdown_idx) => {
                if self.inhibit_next_selection > 0 {
                    self.inhibit_next_selection -= 1;
                    log::debug!("Automatically triggered selection signal inhibited to prevent loop ({} skips left)", self.inhibit_next_selection);
                    return;
                }
                let (_, _, source) = self.possible_sources[dropdown_idx as usize].clone();
                sender.output(OutputRowOutput::SetOutputSource(index.clone(), source));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(ActionOutputsInput::_FromRow(output))
    }
}
