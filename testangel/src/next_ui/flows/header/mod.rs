use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk, Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent,
};
use rust_i18n::t;
use testangel::action_loader::ActionMap;

mod add_step_factory;

#[derive(Debug)]
pub struct FlowsHeader {
    action_map: Arc<ActionMap>,
    add_button: gtk::MenuButton,
    flow_open: bool,
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
    /// Inform the header bar if a flow is open or not.
    ChangeFlowOpen(bool),
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

            #[local_ref]
            add_button -> gtk::MenuButton {
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
                #[watch]
                set_sensitive: model.flow_open,
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::RunFlow).unwrap();
                },
            },
        },

        #[name = "end"]
        gtk::Box {
            set_spacing: 5,

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
    }

    menu! {
        flows_menu: {
            &t!("flows.header.new") => FlowsNewAction,
            &t!("flows.header.open") => FlowsOpenAction,
            &t!("flows.header.save") => FlowsSaveAction,
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
            flow_open: false,
            add_button: gtk::MenuButton::default(),
            search_results: FactoryVecDeque::new(gtk::Box::default(), sender.input_sender()),
        };
        // Reset search results
        sender.input(FlowsHeaderInput::SearchForSteps(String::new()));

        let results_box = model.search_results.widget();
        let add_button = &model.add_button;
        let widgets = view_output!();

        let sender_c = sender.clone();
        let new_action: RelmAction<FlowsNewAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(FlowsHeaderOutput::NewFlow).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<FlowsNewAction>(&["<primary>N"]);

        let sender_c = sender.clone();
        let open_action: RelmAction<FlowsOpenAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(FlowsHeaderOutput::OpenFlow).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<FlowsOpenAction>(&["<primary>O"]);

        let sender_c = sender.clone();
        let save_action: RelmAction<FlowsSaveAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(FlowsHeaderOutput::SaveFlow).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<FlowsSaveAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let save_as_action: RelmAction<FlowsSaveAsAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(FlowsHeaderOutput::SaveAsFlow).unwrap();
        });
        relm4::main_application()
            .set_accelerators_for_action::<FlowsSaveAsAction>(&["<primary><shift>S"]);

        let sender_c = sender.clone();
        let close_action: RelmAction<FlowsCloseAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(FlowsHeaderOutput::CloseFlow).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<FlowsCloseAction>(&["<primary>W"]);

        let sender_c = sender.clone();
        let about_action: RelmAction<FlowsAboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(FlowsHeaderInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<FlowsAboutAction>(&["<primary>A"]);

        let mut group = RelmActionGroup::<FlowsActionGroup>::new();
        group.add_action(new_action);
        group.add_action(open_action);
        group.add_action(save_action);
        group.add_action(save_as_action);
        group.add_action(close_action);
        group.add_action(about_action);
        group.register_for_widget(&widgets.end);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowsHeaderInput::ChangeFlowOpen(now) => {
                self.flow_open = now;
            }
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
                // close popover
                self.add_button.popdown();
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
                            unsorted_results
                                .push((format!("{group}: {}", action.friendly_name), action));
                        }
                    }

                    // Sort
                    unsorted_results.sort_by(|(a, _a), (b, _b)| a.cmp(b));
                    for (_, a) in unsorted_results {
                        results.push_back(a);
                    }
                } else {
                    let mut unsorted_results = vec![];
                    use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
                    let matcher = SkimMatcherV2::default();
                    for (group, actions) in self.action_map.get_by_group() {
                        for action in actions {
                            if let Some(score) = matcher
                                .fuzzy_match(&format!("{group}: {}", action.friendly_name), &query)
                            {
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
relm4::new_stateless_action!(FlowsOpenAction, FlowsActionGroup, "open");
relm4::new_stateless_action!(FlowsSaveAction, FlowsActionGroup, "save");
relm4::new_stateless_action!(FlowsSaveAsAction, FlowsActionGroup, "save-as");
relm4::new_stateless_action!(FlowsCloseAction, FlowsActionGroup, "close");
relm4::new_stateless_action!(FlowsAboutAction, FlowsActionGroup, "about");
