use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};

mod header_bar;

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
    ChangeView(AppView),
}

#[derive(Debug)]
struct AppModel {
    view: AppView,
    header: Controller<header_bar::HeaderBarModel>,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();

    view! {
        main_window = gtk::Window {
            set_title: Some("TestAngel"),
            set_default_width: 800,
            set_default_height: 600,
            set_titlebar: Some(model.header.widget()),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    #[watch]
                    set_label: &format!("Showing: {:?}", model.view)
                },
            }
        }
    }

    fn init(
        init: Self::Init,
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

        let model = AppModel {
            view: AppView::Flows,
            header,
        };
        let widgets = view_output!();
        log::debug!("Initialised model: {model:?}");

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            AppInput::ChangeView(view) => self.view = view,
        }
    }
}
