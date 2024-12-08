use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    adw, factory::FactoryVecDeque, gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt,
    Sender,
};
use testangel::action_loader::ActionMap;

use crate::ui::{
    components::add_step_factory::{AddStepInit, AddStepResult},
    header_bar::HeaderBarInput,
    lang,
};

#[derive(Debug)]
pub struct FlowsHeader {
    action_map: Arc<ActionMap>,
    add_button: gtk::MenuButton,
    flow_open: bool,
    search_results: FactoryVecDeque<AddStepResult>,
    generic_sender: Option<Sender<HeaderBarInput>>,
}

#[derive(Debug)]
pub enum FlowsHeaderOutput {
    NewFlow,
    OpenFlow,
    SaveFlow,
    SaveAsFlow,
    CloseFlow,
    RunFlow,
    AddStep(String),
}

#[derive(Debug)]
pub enum FlowsHeaderInput {
    ActionsMapChanged(Arc<ActionMap>),
    /// Add the step with the action ID given
    AddStep(String),
    /// Trigger a search for the steps provided
    SearchForSteps(String),
    /// Add the top search result to the flow.
    AddTopSearchResult,
    /// Inform the header bar if a flow is open or not.
    ChangeFlowOpen(bool),
    /// Ask this to output the provided event
    PleaseOutput(FlowsHeaderOutput),
    /// Provide this actions header with a sender to update the generic header bar
    SetGenericHeaderBarSender(Sender<HeaderBarInput>),
}

#[relm4::component(pub)]
impl Component for FlowsHeader {
    type Init = Arc<ActionMap>;
    type Input = FlowsHeaderInput;
    type Output = FlowsHeaderOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "start"]
        gtk::Box {
            set_spacing: 5,

            #[local_ref]
            add_button -> gtk::MenuButton {
                set_icon_name: relm4_icons::icon_names::PLUS,
                set_tooltip: &lang::lookup("flow-header-add"),

                #[wrap(Some)]
                #[name = "menu_popover"]
                set_popover = &gtk::Popover {
                    gtk::Box {
                        set_spacing: 2,
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::SearchEntry {
                            set_max_width_chars: 20,

                            connect_activate[sender] => move |_| {
                                sender.input(FlowsHeaderInput::AddTopSearchResult);
                            },

                            connect_search_changed[sender] => move |slf| {
                                let query = slf.text().to_string();
                                sender.input(FlowsHeaderInput::SearchForSteps(query));
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
            gtk::Button {
                set_icon_name: relm4_icons::icon_names::PLAY,
                set_tooltip: &lang::lookup("flow-header-run"),
                #[watch]
                set_sensitive: model.flow_open,
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::RunFlow).unwrap();
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader {
            action_map: init,
            flow_open: false,
            add_button: gtk::MenuButton::default(),
            search_results: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), FlowsHeaderInput::AddStep),
            generic_sender: None,
        };
        // Reset search results
        sender.input(FlowsHeaderInput::SearchForSteps(String::new()));

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
            FlowsHeaderInput::PleaseOutput(output) => {
                let _ = sender.output(output);
            }
            FlowsHeaderInput::ChangeFlowOpen(now) => {
                self.flow_open = now;
                if let Some(gs) = &self.generic_sender {
                    gs.send(HeaderBarInput::FlowOpened(now)).unwrap();
                }
            }
            FlowsHeaderInput::ActionsMapChanged(new_map) => {
                self.action_map = new_map;
                sender.input(FlowsHeaderInput::SearchForSteps(String::new()));
            }
            FlowsHeaderInput::SetGenericHeaderBarSender(generic_sender) => {
                self.generic_sender = Some(generic_sender);
            }
            FlowsHeaderInput::AddStep(step_id) => {
                // close popover
                self.add_button.popdown();
                // unwrap rationale: the receiver will never be disconnected
                sender.output(FlowsHeaderOutput::AddStep(step_id)).unwrap();
            }
            FlowsHeaderInput::AddTopSearchResult => {
                if let Some(result) = self.search_results.get(0) {
                    widgets.menu_popover.popdown();
                    let id = result.value();
                    // unwrap rationale: the receiver will never be disconnected
                    sender.output(FlowsHeaderOutput::AddStep(id)).unwrap();
                }
            }
            FlowsHeaderInput::SearchForSteps(query) => {
                let mut results = self.search_results.guard();
                results.clear();

                // Reset scroll
                let adj = widgets.menu_scrolled_area.vadjustment();
                adj.set_value(adj.lower());

                let show_hidden = std::env::var("TA_SHOW_HIDDEN_ACTIONS")
                    .unwrap_or("no".to_string())
                    .eq_ignore_ascii_case("yes");
                // Collect results
                if query.is_empty() {
                    // List all alphabetically
                    let mut unsorted_results = vec![];
                    for (group, actions) in self.action_map.get_by_group() {
                        for action in actions {
                            if action.visible || show_hidden {
                                unsorted_results
                                    .push((format!("{group}: {}", action.friendly_name), action));
                            }
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _a), (b, _b)| a.cmp(b));
                    for (_, a) in unsorted_results {
                        results.push_back(AddStepInit {
                            label: format!("{}: {}", a.group, a.friendly_name),
                            value: a.id,
                        });
                    }
                } else {
                    let mut unsorted_results = vec![];
                    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
                    let matcher = SkimMatcherV2::default();
                    for (group, actions) in self.action_map.get_by_group() {
                        for action in actions {
                            if action.visible || show_hidden {
                                if let Some(score) = matcher.fuzzy_match(
                                    &format!("{group}: {}", action.friendly_name),
                                    &query,
                                ) {
                                    unsorted_results.push((score, action));
                                }
                            }
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _a), (b, _b)| a.cmp(b));
                    for (_, a) in unsorted_results {
                        results.push_back(AddStepInit {
                            label: format!("{}: {}", a.group, a.friendly_name),
                            value: a.id,
                        });
                    }
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
