use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    adw, factory::FactoryVecDeque, gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};
use testangel::{action_loader::ActionMap, ipc::EngineList};

use crate::ui::{
    components::add_step_factory::{AddStepInit, AddStepResult},
    lang,
};

#[derive(Debug)]
pub struct ActionsHeader {
    action_map: Arc<ActionMap>,
    engine_list: Arc<EngineList>,
    add_button: gtk::MenuButton,
    action_open: bool,
    search_results: FactoryVecDeque<AddStepResult>,
}

#[derive(Debug)]
pub enum ActionsHeaderOutput {
    NewAction,
    OpenAction,
    SaveAction,
    SaveAsAction,
    CloseAction,
    AddStep(String),
}

#[derive(Debug)]
pub enum ActionsHeaderInput {
    ActionsMapChanged(Arc<ActionMap>),
    /// Add the step with the instruction ID given
    AddStep(String),
    /// Trigger a search for the steps provided
    SearchForSteps(String),
    /// Add the top search result to the action.
    AddTopSearchResult,
    /// Inform the header bar if a action is open or not.
    ChangeActionOpen(bool),
    /// Ask this to output the provided event
    PleaseOutput(ActionsHeaderOutput),
}

#[relm4::component(pub)]
impl Component for ActionsHeader {
    type Init = (Arc<EngineList>, Arc<ActionMap>);
    type Input = ActionsHeaderInput;
    type Output = ActionsHeaderOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "start"]
        gtk::Box {
            set_spacing: 5,

            #[local_ref]
            add_button -> gtk::MenuButton {
                set_icon_name: relm4_icons::icon_names::PLUS,
                set_tooltip: &lang::lookup("action-header-add"),

                #[wrap(Some)]
                #[name = "menu_popover"]
                set_popover = &gtk::Popover {
                    gtk::Box {
                        set_spacing: 2,
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::SearchEntry {
                            set_max_width_chars: 20,

                            connect_activate[sender] => move |_| {
                                sender.input(ActionsHeaderInput::AddTopSearchResult);
                            },

                            connect_search_changed[sender] => move |slf| {
                                let query = slf.text().to_string();
                                sender.input(ActionsHeaderInput::SearchForSteps(query));
                            },
                        },

                        #[name = "menu_scrolled_area"]
                        gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            set_min_content_height: 150,

                            #[local_ref]
                            results_box -> gtk::Box {
                                set_spacing: 2,
                                set_orientation: gtk::Orientation::Vertical,
                            },
                        },
                    },
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ActionsHeader {
            engine_list: init.0,
            action_map: init.1,
            action_open: false,
            add_button: gtk::MenuButton::default(),
            search_results: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), ActionsHeaderInput::AddStep),
        };
        // Reset search results
        sender.input(ActionsHeaderInput::SearchForSteps(String::new()));

        let results_box = model.search_results.widget();
        let add_button = &model.add_button;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ActionsHeaderInput::PleaseOutput(output) => {
                let _ = sender.output(output);
            }
            ActionsHeaderInput::ChangeActionOpen(now) => {
                self.action_open = now;
            }
            ActionsHeaderInput::ActionsMapChanged(new_map) => {
                self.action_map = new_map;
            }
            ActionsHeaderInput::AddStep(step_id) => {
                // close popover
                self.add_button.popdown();
                // unwrap rationale: the receiver will never be disconnected
                sender
                    .output(ActionsHeaderOutput::AddStep(step_id))
                    .unwrap();
            }
            ActionsHeaderInput::AddTopSearchResult => {
                if let Some(result) = self.search_results.get(0) {
                    widgets.menu_popover.popdown();
                    let id = result.value();
                    // unwrap rationale: the receiver will never be disconnected
                    sender.output(ActionsHeaderOutput::AddStep(id)).unwrap();
                }
            }
            ActionsHeaderInput::SearchForSteps(query) => {
                let mut results = self.search_results.guard();
                results.clear();

                // Reset scroll
                let adj = widgets.menu_scrolled_area.vadjustment();
                adj.set_value(adj.lower());

                // Collect results
                if query.is_empty() {
                    // List all alphabetically
                    let mut unsorted_results = vec![];
                    for engine in self.engine_list.inner() {
                        for instruction in &engine.instructions {
                            unsorted_results.push((
                                format!("{}: {}", engine.name, instruction.friendly_name()),
                                engine.name.clone(),
                                instruction.clone(),
                            ));
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
                    for (_, engine_name, ins) in unsorted_results {
                        results.push_back(AddStepInit {
                            label: format!("{engine_name}: {}", ins.friendly_name()),
                            value: ins.id().clone(),
                        });
                    }
                } else {
                    let mut unsorted_results = vec![];
                    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
                    let matcher = SkimMatcherV2::default();
                    for engine in self.engine_list.inner() {
                        for instruction in &engine.instructions {
                            if let Some(score) = matcher.fuzzy_match(
                                &format!("{}: {}", engine.name, instruction.friendly_name()),
                                &query,
                            ) {
                                unsorted_results.push((
                                    score,
                                    engine.name.clone(),
                                    instruction.clone(),
                                ));
                            }
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
                    for (_, engine_name, ins) in unsorted_results {
                        results.push_back(AddStepInit {
                            label: format!("{engine_name}: {}", ins.friendly_name()),
                            value: ins.id().clone(),
                        });
                    }
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
