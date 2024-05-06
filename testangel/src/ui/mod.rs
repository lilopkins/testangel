use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{
    actions::RelmActionGroup,
    adw,
    gtk::{self, ApplicationInhibitFlags},
    Component, ComponentController, ComponentParts, Controller, RelmApp,
};
use testangel::{
    action_loader::{self, ActionMap},
    ipc::{self, EngineList},
};

use self::header_bar::HeaderBarInput;

mod about;
mod actions;
mod components;
mod file_filters;
mod flows;
mod header_bar;
pub(crate) mod lang;

/// Initialise and open the UI.
pub fn initialise_ui() {
    log::info!("Starting Next UI...");
    let app = RelmApp::new("uk.hpkns.testangel");
    relm4_icons::initialize_icons();

    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/uk/hpkns/testangel/icons/");
    theme.add_resource_path("/uk/hpkns/testangel/icons/scalable/actions/");

    let engines = Arc::new(ipc::get_engines());
    let actions = Arc::new(action_loader::get_actions(engines.clone()));
    app.run::<AppModel>(AppInit { engines, actions });
}

pub struct AppInit {
    engines: Arc<EngineList>,
    actions: Arc<ActionMap>,
}

#[derive(Debug)]
enum AppInput {
    /// The view has changed and should be read from visible_child_name, then components updated as needed.
    ChangedView(Option<String>),
    /// The actions might have changed and should be reloaded
    ReloadActionsMap,
    /// Attach the action group to the window
    AttachGeneralActionGroup(RelmActionGroup<header_bar::GeneralActionGroup>),
    /// Attach the action group to the window
    AttachFileActionGroup(RelmActionGroup<header_bar::FileActionGroup>),
    /// Request that any open files are saved.
    SafeCloseAllFiles,
    /// Update whether a flow is currently open.
    UpdateFlowOpen(bool),
    /// Update whether an action is currently open.
    UpdateActionOpen(bool),
    /// Signal that this window should close ASAP.
    CloseWhenAble,
}

#[derive(Debug)]
struct AppModel {
    stack: Rc<adw::ViewStack>,
    header: Controller<header_bar::HeaderBarModel>,

    flows: Controller<flows::FlowsModel>,
    actions: Controller<actions::ActionsModel>,

    engines_list: Arc<EngineList>,
    actions_map: Arc<ActionMap>,

    close_when_able: bool,
    flow_open: bool,
    action_open: bool,
    inhibit_cookie: Option<u32>,
}

#[relm4::component]
impl Component for AppModel {
    type Init = AppInit;
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        main_window = adw::Window {
            set_title: Some(&lang::lookup("app-name")),
            set_default_width: 800,
            set_default_height: 600,
            set_icon_name: Some(relm4_icons::icon_names::TESTANGEL),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                model.header.widget(),

                #[local_ref]
                stack -> adw::ViewStack {
                    connect_visible_child_name_notify[sender] => move |st| {
                        sender.input(AppInput::ChangedView(st.visible_child_name().map(|s| s.into())));
                    },
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        // Initialise the sub-components (pages)
        let flows = flows::FlowsModel::builder()
            .launch((init.actions.clone(), init.engines.clone()))
            .forward(sender.input_sender(), |msg| match msg {
                flows::FlowOutputs::FileState(open) => AppInput::UpdateFlowOpen(open),
            });
        let actions = actions::ActionsModel::builder()
            .launch((init.actions.clone(), init.engines.clone()))
            .forward(sender.input_sender(), |msg| match msg {
                actions::ActionOutputs::ReloadActions => AppInput::ReloadActionsMap,
                actions::ActionOutputs::FileState(open) => AppInput::UpdateActionOpen(open),
            });

        let stack = Rc::new(adw::ViewStack::new());
        gtk::Window::set_default_icon_name("testangel");

        // Initialise the headerbar
        let header = header_bar::HeaderBarModel::builder()
            .launch((
                actions.model().header_controller_rc(),
                flows.model().header_controller_rc(),
                stack.clone(),
                init.engines.clone(),
                init.actions.clone(),
            ))
            .forward(sender.input_sender(), |msg| match msg {
                header_bar::HeaderBarOutput::AttachGeneralActionGroup(group) => {
                    AppInput::AttachGeneralActionGroup(group)
                }
                header_bar::HeaderBarOutput::AttachFileActionGroup(group) => {
                    AppInput::AttachFileActionGroup(group)
                }
            });

        // Build model
        let model = AppModel {
            actions_map: init.actions,
            engines_list: init.engines,
            stack,
            header,
            flows,
            actions,
            close_when_able: false,
            flow_open: false,
            action_open: false,
            inhibit_cookie: None,
        };

        // Render window parts
        let stack = &*model.stack;

        // Add pages
        stack.add_titled_with_icon(
            model.flows.widget(),
            Some("flows"),
            &lang::lookup("tab-flows"),
            relm4_icons::icon_names::PAPYRUS_VERTICAL,
        );
        if !std::env::var("TA_HIDE_ACTION_EDITOR")
            .unwrap_or("no".to_string())
            .eq_ignore_ascii_case("yes")
        {
            stack.add_titled_with_icon(
                model.actions.widget(),
                Some("actions"),
                &lang::lookup("tab-actions"),
                relm4_icons::icon_names::PUZZLE_PIECE,
            );
        }

        let widgets = view_output!();
        log::debug!("Initialised model: {model:?}");

        // Trigger initial header bar update
        sender.input(AppInput::ChangedView(
            stack.visible_child_name().map(|s| s.into()),
        ));

        // Track state to intervene on window close
        let app = relm4::main_application();
        app.set_register_session(true);
        let sender_c = sender.clone();
        app.connect_query_end(move |_app| {
            // Ask everything to save. Will be inhibited if anything is open.
            sender_c.input(AppInput::SafeCloseAllFiles);
        });
        let sender_c = sender.clone();
        widgets.main_window.connect_close_request(move |_win| {
            // Ask everything to save. Will be inhibited if anything is open.
            sender_c.input(AppInput::SafeCloseAllFiles);
            sender_c.input(AppInput::CloseWhenAble);
            gtk::glib::Propagation::Stop
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            AppInput::AttachGeneralActionGroup(group) => {
                group.register_for_widget(root);
            }
            AppInput::AttachFileActionGroup(group) => {
                group.register_for_widget(root);
            }
            AppInput::ChangedView(new_view) => {
                self.header
                    .emit(HeaderBarInput::ChangedView(new_view.unwrap_or_default()));
            }
            AppInput::ReloadActionsMap => {
                self.actions_map = Arc::new(action_loader::get_actions(self.engines_list.clone()));
                self.flows.emit(flows::FlowInputs::ActionsMapChanged(
                    self.actions_map.clone(),
                ));
                self.actions.emit(actions::ActionInputs::ActionsMapChanged(
                    self.actions_map.clone(),
                ));
                self.header
                    .emit(header_bar::HeaderBarInput::ActionsMapChanged(
                        self.actions_map.clone(),
                    ))
            }
            AppInput::SafeCloseAllFiles => {
                self.flows.emit(flows::FlowInputs::CloseFlow);
                self.actions.emit(actions::ActionInputs::CloseAction);
            }
            AppInput::CloseWhenAble => {
                self.close_when_able = true;
            }
            AppInput::UpdateFlowOpen(open) => {
                self.flow_open = open;
                if self.flow_open || self.action_open {
                    if self.inhibit_cookie.is_none() {
                        // Needs inhibiting
                        self.inhibit_cookie = Some(relm4::main_application().inhibit(
                            None::<&relm4::gtk::Window>,
                            ApplicationInhibitFlags::LOGOUT,
                            Some(&lang::lookup("files-need-saving")),
                        ));
                    }
                } else {
                    if let Some(cookie) = self.inhibit_cookie {
                        // Needs uninhibiting
                        relm4::main_application().uninhibit(cookie);
                    }
                    if self.close_when_able {
                        // Close now!
                        relm4::main_application().quit();
                    }
                }
                log::debug!(
                    "Close inhibit cookie is {}!",
                    if self.inhibit_cookie.is_none() {
                        "unset"
                    } else {
                        "set"
                    }
                );
            }
            AppInput::UpdateActionOpen(open) => {
                self.action_open = open;
                if self.flow_open || self.action_open {
                    if self.inhibit_cookie.is_none() {
                        // Needs inhibiting
                        self.inhibit_cookie = Some(relm4::main_application().inhibit(
                            None::<&relm4::gtk::Window>,
                            ApplicationInhibitFlags::LOGOUT,
                            Some(&lang::lookup("files-need-saving")),
                        ));
                    }
                } else {
                    if let Some(cookie) = self.inhibit_cookie {
                        // Needs uninhibiting
                        relm4::main_application().uninhibit(cookie);
                    }
                    if self.close_when_able {
                        // Close now!
                        relm4::main_application().quit();
                    }
                }
                log::debug!(
                    "Close inhibit cookie is {}!",
                    if self.inhibit_cookie.is_none() {
                        "unset"
                    } else {
                        "set"
                    }
                );
            }
        }
    }
}
