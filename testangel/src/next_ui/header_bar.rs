use std::rc::Rc;

use gtk::prelude::*;
use relm4::{
    gtk, ComponentController, ComponentParts, ComponentSender, Controller, RelmIterChildrenExt,
    SimpleComponent,
};
use rust_i18n::t;

use super::flows::FlowsHeader;

#[derive(Debug)]
pub struct HeaderBarModel {
    start_box: gtk::Box,
    actions_enabled: bool,

    flows_header: Rc<Controller<FlowsHeader>>,
}

impl HeaderBarModel {
    fn change_start_box(&mut self, new_box: &gtk::Box) {
        for child in self.start_box.iter_children() {
            self.start_box.remove(&child);
        }
        self.start_box.append(new_box);
    }
}

#[derive(Debug)]
pub enum HeaderBarInput {
    ViewChanged(super::AppView),
}

#[derive(Debug)]
pub enum HeaderBarOutput {
    Flows,
    Actions,
    Help,
}

#[relm4::component(pub)]
impl SimpleComponent for HeaderBarModel {
    type Init = Rc<Controller<FlowsHeader>>;
    type Input = HeaderBarInput;
    type Output = HeaderBarOutput;

    view! {
        #[root]
        gtk::HeaderBar {
            #[local_ref]
            pack_start = start_box -> gtk::Box,

            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",

                #[name = "group"]
                gtk::ToggleButton {
                    set_label: &t!("header.flows"),
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderBarOutput::Flows).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: &t!("header.actions"),
                    set_group: Some(&group),
                    set_visible: model.actions_enabled,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(HeaderBarOutput::Actions).unwrap()
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: &t!("header.help"),
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
            start_box: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            actions_enabled: !std::env::var("TA_HIDE_ACTION_EDITOR")
                .unwrap_or("no".to_string())
                .eq_ignore_ascii_case("yes"),

            flows_header: init,
        };

        let start_box = &model.start_box;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            HeaderBarInput::ViewChanged(new_view) => {
                match new_view {
                    super::AppView::Flows => {
                        let header = self.flows_header.clone();
                        self.change_start_box(header.widget());
                    }
                    _ => {
                        // Clear header box
                        self.change_start_box(&gtk::Box::builder().build());
                    }
                }
            }
        }
    }
}
