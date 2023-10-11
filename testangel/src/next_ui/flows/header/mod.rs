use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent, factory::FactoryVecDeque,
};
use rust_i18n::t;
use testangel::action_loader::ActionMap;

mod add_step_factory;

#[derive(Debug)]
pub struct FlowsHeader {
    action_map: Arc<ActionMap>,
    search_results: FactoryVecDeque<add_step_factory::StepSearchResult>,
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
    OpenAboutDialog,
    ActionsMapChanged(Arc<ActionMap>),
    /// Add the step with the action ID given
    AddStep(String),
    /// Trigger a search for the steps provided
    SearchForSteps(String),
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsHeader {
    type Init = Arc<ActionMap>;
    type Input = FlowsHeaderInput;
    type Output = FlowsHeaderOutput;

    view! {
        #[root]
        #[name = "start"]
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_label: &t!("open"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::OpenFlow).unwrap();
                },
            },
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::PLUS,
                set_tooltip: &t!("flows.header.add"),

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    gtk::Box {
                        set_spacing: 2,
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::SearchEntry {
                            set_max_width_chars: 20,

                            connect_search_changed[sender] => move |slf| {
                                let query = slf.text().to_string();
                                sender.input(FlowsHeaderInput::SearchForSteps(query));
                            },
                        },

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
                set_icon_name: relm4_icons::icon_name::PLAY,
                set_tooltip: &t!("flows.header.run"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::RunFlow).unwrap();
                },
            },
        },

        #[name = "end"]
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_label: &t!("save"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::SaveFlow).unwrap();
                },
            },
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &t!("flows.header.more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&flows_menu)) {
                    set_position: gtk::PositionType::Bottom,
                },
            },
        },

        menu_save_widget = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            #[name = "group"]
            gtk::ToggleButton {
                set_label: "O",
            },
            gtk::ToggleButton {
                set_label: "S",
                set_group: Some(&group),
            },
            gtk::ToggleButton {
                set_label: "SA",
                set_group: Some(&group),
            },
        },
    }

    menu! {
        flows_menu: {
            custom: "menu_save_widget",
            &t!("flows.header.new") => FlowsNewAction,
            &t!("flows.header.save-as") => FlowsSaveAsAction,
            &t!("flows.header.close") => FlowsCloseAction,
            section! {
                &t!("header.about") => FlowsAboutAction,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader {
            action_map: init,
            search_results: FactoryVecDeque::new(gtk::Box::default(), sender.input_sender()),
        };
        // Reset search results
        sender.input(FlowsHeaderInput::SearchForSteps(String::new()));

        let results_box = model.search_results.widget();
        let widgets = view_output!();

        let about_action: RelmAction<FlowsAboutAction> = RelmAction::new_stateless(move |_| {
            sender.input(FlowsHeaderInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<FlowsAboutAction>(&["<primary>A"]);

        let mut group = RelmActionGroup::<FlowsActionGroup>::new();
        group.add_action(about_action);
        group.register_for_widget(&widgets.end);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowsHeaderInput::OpenAboutDialog => {
                crate::next_ui::about::AppAbout::builder()
                    // TODO .transient_for()
                    .launch(())
                    .widget()
                    .show();
            }
            FlowsHeaderInput::ActionsMapChanged(new_map) => {
                self.action_map = new_map;
            }
            FlowsHeaderInput::AddStep(step_id) => {
                // unwrap rationale: the receiver will never be disconnected
                sender.output(FlowsHeaderOutput::AddStep(step_id)).unwrap();
            }
            FlowsHeaderInput::SearchForSteps(query) => {
                let mut results = self.search_results.guard();
                results.clear();

                // Collect results
                if query.is_empty() {
                    // List all alphabetically
                    let mut unsorted_results = vec![];
                    for (group, actions) in self.action_map.get_by_group() {
                        for action in actions {
                            unsorted_results.push((format!("{group}: {}", action.friendly_name), action));
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _a), (b, _b)| a.cmp(b));
                    for (_, a) in unsorted_results {
                        results.push_back(a);
                    }
                } else {
                    let mut unsorted_results = vec![];
                    use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
                    let matcher = SkimMatcherV2::default();
                    for (group, actions) in self.action_map.get_by_group() {
                        for action in actions {
                            if let Some(score) = matcher.fuzzy_match(&format!("{group}: {}", action.friendly_name), &query) {
                                unsorted_results.push((score, action));
                            }
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _a), (b, _b)| a.cmp(b));
                    for (_, a) in unsorted_results {
                        results.push_back(a);
                    }
                }
            }
        }
    }
}

relm4::new_action_group!(FlowsActionGroup, "flows");
relm4::new_stateless_action!(FlowsNewAction, FlowsActionGroup, "new");
relm4::new_stateless_action!(FlowsSaveAsAction, FlowsActionGroup, "save-as");
relm4::new_stateless_action!(FlowsCloseAction, FlowsActionGroup, "close");
relm4::new_stateless_action!(FlowsAboutAction, FlowsActionGroup, "about");
