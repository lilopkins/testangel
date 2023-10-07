use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, Controller, RelmApp, SimpleComponent,
};
use rust_i18n::t;
use testangel::{
    action_loader::{self, ActionMap},
    ipc::{self, EngineList},
};

use self::header_bar::HeaderBarInput;

mod about;
mod actions;
mod flows;
mod header_bar;
mod help;

/// Initialise and open the UI.
pub fn initialise_ui() {
    log::info!("Starting Next UI...");
    let app = RelmApp::new("lilopkins.testangel");
    relm4_icons::initialize_icons();

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
    NoOp,
    /// The view has changed and should be read from visible_child_name, then components updated as needed.
    ChangedView(Option<String>),
}

#[derive(Debug)]
struct AppModel {
    stack: Rc<adw::ViewStack>,
    header: Controller<header_bar::HeaderBarModel>,

    flows: Controller<flows::FlowsModel>,
    actions: Controller<actions::ActionsModel>,

    engines_list: Arc<EngineList>,
    actions_map: Arc<ActionMap>,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = AppInit;
    type Input = AppInput;
    type Output = ();

    view! {
        main_window = adw::Window {
            set_title: Some(&t!("name")),
            set_default_width: 800,
            set_default_height: 600,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                model.header.widget(),

                #[local_ref]
                stack -> adw::ViewStack {
                    add_titled_with_icon[Some("flows"), &t!("header.flows"), relm4_icons::icon_name::PAPYRUS_VERTICAL] = model.flows.widget(),
                    add_titled_with_icon[Some("actions"), &t!("header.actions"), relm4_icons::icon_name::PUZZLE_PIECE] = model.actions.widget(),

                    connect_visible_child_name_notify[sender] => move |st| {
                        sender.input(AppInput::ChangedView(st.visible_child_name().map(|s| s.into())));
                    },
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        // Initialise the sub-components (pages)
        let flows = flows::FlowsModel::builder()
            .launch((
                gtk::FileChooserDialog::builder()
                    .transient_for(root)
                    .build(),
                gtk::FileChooserDialog::builder()
                    .transient_for(root)
                    .build(),
                init.actions.clone(),
                init.engines.clone(),
            ))
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);
        let actions = actions::ActionsModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);

        let stack = Rc::new(adw::ViewStack::new());

        // Initialise the headerbar
        let header = header_bar::HeaderBarModel::builder()
            .launch((flows.model().header_controller_rc(), stack.clone()))
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);

        // Build model
        let model = AppModel {
            actions_map: init.actions,
            engines_list: init.engines,
            stack,
            header,
            flows,
            actions,
        };

        // Render window parts
        let stack = &*model.stack;
        let widgets = view_output!();
        log::debug!("Initialised model: {model:?}");

        // Trigger initial header bar update
        sender.input(AppInput::ChangedView(
            stack.visible_child_name().map(|s| s.into()),
        ));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match message {
            AppInput::NoOp => (),
            AppInput::ChangedView(new_view) => {
                self.header
                    .emit(HeaderBarInput::ChangedView(new_view.unwrap_or_default()));
            }
        }
    }
}
