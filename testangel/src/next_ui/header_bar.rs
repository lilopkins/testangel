use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Debug)]
pub struct HeaderBarModel {
    actions_enabled: bool,
}

#[derive(Debug)]
pub enum HeaderBarOutput {
    Flows,
    Actions,
    Help,
}

#[relm4::component(pub)]
impl SimpleComponent for HeaderBarModel {
    type Init = ();
    type Input = ();
    type Output = HeaderBarOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",

                #[name = "group"]
                gtk::ToggleButton {
                    set_label: "Flows",
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderBarOutput::Flows).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Actions",
                    set_group: Some(&group),
                    #[watch]
                    set_visible: model.actions_enabled,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderBarOutput::Actions).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Help",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderBarOutput::Help).unwrap()
                        }
                    },
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderBarModel {
            actions_enabled: !std::env::var("TA_HIDE_ACTION_EDITOR")
                .unwrap_or("no".to_string())
                .eq_ignore_ascii_case("yes"),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}