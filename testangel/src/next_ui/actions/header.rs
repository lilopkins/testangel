use std::sync::Arc;

use adw::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw,
    factory::FactoryVecDeque,
    gtk, Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
};
use testangel::{action_loader::ActionMap, ipc::EngineList};

use crate::next_ui::{lang, components::add_step_factory::{AddStepResult, AddStepTrait, AddStepInit}};

#[derive(Debug)]
pub struct ActionsHeader {
    action_map: Arc<ActionMap>,
    engine_list: Arc<EngineList>,
    add_button: gtk::MenuButton,
    action_open: bool,
    search_results: FactoryVecDeque<AddStepResult<ActionsHeaderInput>>,
}

#[derive(Debug)]
pub enum ActionsHeaderOutput {
    NewAction,
    OpenAction,
    SaveAction,
    SaveAsAction,
    CloseAction,
    RunAction,
    AddStep(String),
}

#[derive(Debug)]
pub enum ActionsHeaderInput {
    OpenAboutDialog,
    ActionsMapChanged(Arc<ActionMap>),
    /// Add the step with the instruction ID given
    AddStep(String),
    /// Trigger a search for the steps provided
    SearchForSteps(String),
    /// Add the top search result to the action.
    AddTopSearchResult,
    /// Inform the header bar if a action is open or not.
    ChangeActionOpen(bool),
}

impl AddStepTrait for ActionsHeaderInput {
    fn add_step(value: String) -> Self {
        Self::AddStep(value)
    }
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
                set_icon_name: relm4_icons::icon_name::PLUS,
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
                set_tooltip: &lang::lookup("action-header-run"),
                // TODO uncomment when execution dialog ready
                //#[watch]
                //set_sensitive: model.action_open,
                set_sensitive: false,
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(ActionsHeaderOutput::RunAction).unwrap();
                },
            },
        },

        #[name = "end"]
        gtk::Box {
            set_spacing: 5,

            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &lang::lookup("action-header-more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&actions_menu)) {
                    set_position: gtk::PositionType::Bottom,
                },
            },
        },
    }

    menu! {
        actions_menu: {
            &lang::lookup("action-header-new") => ActionsNewAction,
            &lang::lookup("action-header-open") => ActionsOpenAction,
            &lang::lookup("action-header-save") => ActionsSaveAction,
            &lang::lookup("action-header-save-as") => ActionsSaveAsAction,
            &lang::lookup("action-header-close") => ActionsCloseAction,
            section! {
                &lang::lookup("action-header-about") => ActionsAboutAction,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ActionsHeader {
            engine_list: init.0,
            action_map: init.1,
            action_open: false,
            add_button: gtk::MenuButton::default(),
            search_results: FactoryVecDeque::new(gtk::Box::default(), sender.input_sender()),
        };
        // Reset search results
        sender.input(ActionsHeaderInput::SearchForSteps(String::new()));

        let results_box = model.search_results.widget();
        let add_button = &model.add_button;
        let widgets = view_output!();

        let sender_c = sender.clone();
        let new_action: RelmAction<ActionsNewAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(ActionsHeaderOutput::NewAction).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<ActionsNewAction>(&["<primary>N"]);

        let sender_c = sender.clone();
        let open_action: RelmAction<ActionsOpenAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(ActionsHeaderOutput::OpenAction).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<ActionsOpenAction>(&["<primary>O"]);

        let sender_c = sender.clone();
        let save_action: RelmAction<ActionsSaveAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(ActionsHeaderOutput::SaveAction).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<ActionsSaveAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let save_as_action: RelmAction<ActionsSaveAsAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(ActionsHeaderOutput::SaveAsAction).unwrap();
        });
        relm4::main_application()
            .set_accelerators_for_action::<ActionsSaveAsAction>(&["<primary><shift>S"]);

        let sender_c = sender.clone();
        let close_action: RelmAction<ActionsCloseAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.output(ActionsHeaderOutput::CloseAction).unwrap();
        });
        relm4::main_application().set_accelerators_for_action::<ActionsCloseAction>(&["<primary>W"]);

        let sender_c = sender.clone();
        let about_action: RelmAction<ActionsAboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(ActionsHeaderInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<ActionsAboutAction>(&["<primary>A"]);

        let mut group = RelmActionGroup::<ActionsActionGroup>::new();
        group.add_action(new_action);
        group.add_action(open_action);
        group.add_action(save_action);
        group.add_action(save_as_action);
        group.add_action(close_action);
        group.add_action(about_action);
        group.register_for_widget(&widgets.end);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            ActionsHeaderInput::ChangeActionOpen(now) => {
                self.action_open = now;
            }
            ActionsHeaderInput::ActionsMapChanged(new_map) => {
                self.action_map = new_map;
            }
            ActionsHeaderInput::OpenAboutDialog => {
                crate::next_ui::about::AppAbout::builder()
                    .transient_for(root)
                    .launch((self.engine_list.clone(), self.action_map.clone()))
                    .widget()
                    .set_visible(true);
            }
            ActionsHeaderInput::AddStep(step_id) => {
                // close popover
                self.add_button.popdown();
                // unwrap rationale: the receiver will never be disconnected
                sender.output(ActionsHeaderOutput::AddStep(step_id)).unwrap();
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

                // Collect results
                if query.is_empty() {
                    // List all alphabetically
                    let mut unsorted_results = vec![];
                    for engine in self.engine_list.inner() {
                        for instruction in &engine.instructions {
                            unsorted_results
                                .push((format!("{}: {}", engine.name, instruction.friendly_name()), engine.name.clone(), instruction.clone()));
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
                                unsorted_results
                                    .push((score, engine.name.clone(), instruction.clone()));
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

relm4::new_action_group!(ActionsActionGroup, "actions");
relm4::new_stateless_action!(ActionsNewAction, ActionsActionGroup, "new");
relm4::new_stateless_action!(ActionsOpenAction, ActionsActionGroup, "open");
relm4::new_stateless_action!(ActionsSaveAction, ActionsActionGroup, "save");
relm4::new_stateless_action!(ActionsSaveAsAction, ActionsActionGroup, "save-as");
relm4::new_stateless_action!(ActionsCloseAction, ActionsActionGroup, "close");
relm4::new_stateless_action!(ActionsAboutAction, ActionsActionGroup, "about");
