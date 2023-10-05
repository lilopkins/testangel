use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, Controller, RelmApp, RelmIterChildrenExt,
    SimpleComponent,
};
use rust_i18n::t;

mod actions;
mod flows;
mod header_bar;
mod help;

/// Initialise and open the UI.
pub fn initialise_ui() {
    log::info!("Starting Next UI...");
    let app = RelmApp::new("lilopkins.testangel");
    app.run::<AppModel>(());
}

#[derive(Debug)]
enum AppView {
    Flows,
    Actions,
    Help,
}

#[derive(Debug)]
enum AppInput {
    NoOp,
    ChangeView(AppView),
}

#[derive(Debug)]
struct AppModel {
    view: AppView,
    child_view: gtk::Box,
    header: Controller<header_bar::HeaderBarModel>,

    flows: Controller<flows::FlowsModel>,
    actions: Controller<actions::ActionsModel>,
    help: Controller<help::HelpModel>,
}

impl AppModel {
    fn update_child_view(&mut self) {
        for child in self.child_view.iter_children() {
            self.child_view.remove(&child);
        }
        self.child_view.append(match self.view {
            AppView::Flows => self.flows.widget(),
            AppView::Actions => self.actions.widget(),
            AppView::Help => self.help.widget(),
        });
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();

    view! {
        main_window = gtk::Window {
            set_title: Some(&t!("name")),
            set_default_width: 800,
            set_default_height: 600,
            set_titlebar: Some(model.header.widget()),

            #[local_ref]
            child_view -> gtk::Box { },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let header = header_bar::HeaderBarModel::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                header_bar::HeaderBarOutput::Flows => AppInput::ChangeView(AppView::Flows),
                header_bar::HeaderBarOutput::Actions => AppInput::ChangeView(AppView::Actions),
                header_bar::HeaderBarOutput::Help => AppInput::ChangeView(AppView::Help),
            },
        );

        let flows = flows::FlowsModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);
        let actions = actions::ActionsModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);
        let help = help::HelpModel::builder()
            .launch(())
            .forward(sender.input_sender(), |_msg| AppInput::NoOp);

        let mut model = AppModel {
            view: AppView::Flows,
            child_view: gtk::Box::new(gtk::Orientation::Vertical, 0),
            header,
            flows,
            actions,
            help,
        };
        model.update_child_view();

        let child_view = &model.child_view;
        let widgets = view_output!();
        log::debug!("Initialised model: {model:?}");

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match message {
            AppInput::NoOp => (),
            AppInput::ChangeView(view) => {
                // Change tracked view
                self.view = view;
                // Change frame
                self.update_child_view();
            }
        }
    }
}
