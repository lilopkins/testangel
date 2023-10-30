use adw::prelude::*;
use relm4::{
    adw,
    factory::FactoryVecDeque,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    Component, RelmWidgetExt,
};
use testangel::types::Action;
use testangel_ipc::prelude::ParameterKind;

use crate::next_ui::lang;

#[derive(Debug)]
pub enum ActionParamsInput {
    /// Inform the metadata component that the action has changed and as such
    /// it should reload the metadata values
    ChangeAction(Action),
    /// Create a new parameter
    NewParameter,
    _FromRow(ParamRowOutput),
}

#[derive(Clone, Debug)]
pub enum ActionParamsOutput {
    /// Set parameters
    SetParameters(Vec<(String, ParameterKind)>),
    /// Remove references to the provided index, or reduce any higher than.
    IndexRemoved(usize),
    /// Swap references to the indexes provided
    IndexesSwapped(usize, usize),
}

#[derive(Debug)]
pub struct ActionParams {
    raw_params: Vec<(String, ParameterKind)>,
    params: FactoryVecDeque<ParamRow>,
}

#[relm4::component(pub)]
impl Component for ActionParams {
    type Init = ();
    type Input = ActionParamsInput;
    type Output = ActionParamsOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Button {
                set_label: &lang::lookup("action-params-new"),
                connect_clicked => ActionParamsInput::NewParameter,
            },

            model.params.widget(),
        },
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ActionParams {
            raw_params: vec![],
            params: FactoryVecDeque::new(
                gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(5)
                    .build(),
                sender.input_sender(),
            ),
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
            ActionParamsInput::ChangeAction(action) => {
                let mut params = self.params.guard();
                params.clear();
                self.raw_params.clear();

                // Add each param from action
                for (name, kind) in action.parameters {
                    self.raw_params.push((name.clone(), kind));
                    params.push_back(Some((name, kind)));
                }
            }

            ActionParamsInput::NewParameter => {
                let mut params = self.params.guard();
                params.push_back(Some((String::new(), ParameterKind::String)));
                self.raw_params.push((String::new(), ParameterKind::String));
            }

            ActionParamsInput::_FromRow(actions_output) => match actions_output {
                ParamRowOutput::MoveUp(index) => {
                    let idx = index.current_index();
                    if idx == 0 {
                        return;
                    }
                    let mut params = self.params.guard();
                    self.raw_params.swap(idx, idx - 1);
                    params.swap(idx, idx - 1);
                    let _ = sender.output(ActionParamsOutput::IndexesSwapped(idx, idx - 1));
                }
                ParamRowOutput::MoveDown(index) => {
                    let idx = index.current_index();
                    if idx == self.raw_params.len() - 1 {
                        return;
                    }
                    let mut params = self.params.guard();
                    self.raw_params.swap(idx, idx + 1);
                    params.swap(idx, idx + 1);
                    let _ = sender.output(ActionParamsOutput::IndexesSwapped(idx, idx + 1));
                }
                ParamRowOutput::Delete(index) => {
                    let mut params = self.params.guard();
                    self.raw_params.remove(index.current_index());
                    params.remove(index.current_index());
                    let _ =
                        sender.output(ActionParamsOutput::SetParameters(self.raw_params.clone()));

                    let old_idx = index.current_index();
                    let _ = sender.output(ActionParamsOutput::IndexRemoved(old_idx));
                }
                ParamRowOutput::SetParamKind(index, new_kind) => {
                    self.raw_params[index.current_index()].1 = new_kind;
                    let _ =
                        sender.output(ActionParamsOutput::SetParameters(self.raw_params.clone()));
                }
                ParamRowOutput::SetParamName(index, new_name) => {
                    self.raw_params[index.current_index()].0 = new_name;
                    let _ =
                        sender.output(ActionParamsOutput::SetParameters(self.raw_params.clone()));
                }
            },
        }

        self.update_view(widgets, sender)
    }
}

#[derive(Debug)]
struct ParamRow {
    name: String,
    kind_index: u32,
}

#[derive(Clone, Debug)]
pub enum ParamRowOutput {
    SetParamName(DynamicIndex, String),
    SetParamKind(DynamicIndex, ParameterKind),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Delete(DynamicIndex),
}

static PARAM_KINDS: once_cell::sync::Lazy<Vec<(&'static str, ParameterKind)>> =
    once_cell::sync::Lazy::new(|| {
        vec![
            ("String", ParameterKind::String),
            ("Integer", ParameterKind::Integer),
            ("Decimal", ParameterKind::Decimal),
            ("Boolean", ParameterKind::Boolean),
        ]
    });

#[relm4::factory]
impl FactoryComponent for ParamRow {
    type Init = Option<(String, ParameterKind)>;
    type Input = ();
    type Output = ParamRowOutput;
    type CommandOutput = ();
    type ParentInput = ActionParamsInput;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            // name
            gtk::Entry {
                set_hexpand: true,
                set_text: &self.name,

                connect_changed[sender, index] => move |entry| {
                    sender.output(ParamRowOutput::SetParamName(index.clone(), entry.text().to_string()));
                },
            },

            // kind
            gtk::DropDown {
                set_model: Some(&gtk::StringList::new(PARAM_KINDS.iter().map(|(label, _)| *label).collect::<Vec<_>>().as_slice())),
                set_selected: self.kind_index,

                connect_selected_notify[sender, index] => move |dropdown| {
                    let idx = dropdown.selected();
                    let (_, kind) = PARAM_KINDS[idx as usize];
                    sender.output(ParamRowOutput::SetParamKind(index.clone(), kind));
                },
            },

            // move up
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::UP,
                set_tooltip: &lang::lookup("move-up"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(ParamRowOutput::MoveUp(index.clone()));
                }
            },
            // move down
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::DOWN,
                set_tooltip: &lang::lookup("move-down"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(ParamRowOutput::MoveDown(index.clone()));
                }
            },
            // delete
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::X_CIRCULAR,
                set_tooltip: &lang::lookup("delete"),
                connect_clicked[index, sender] => move |_| {
                    sender.output(ParamRowOutput::Delete(index.clone()));
                }
            }
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        if let Some((name, p_kind)) = init {
            let mut kind_index = gtk::INVALID_LIST_POSITION;
            for (idx, (_, kind)) in PARAM_KINDS.iter().enumerate() {
                if *kind == p_kind {
                    kind_index = idx as u32;
                    break;
                }
            }

            Self { name, kind_index }
        } else {
            Self {
                name: String::new(),
                kind_index: gtk::INVALID_LIST_POSITION,
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(ActionParamsInput::_FromRow(output))
    }
}
