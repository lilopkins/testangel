use std::rc::Rc;

use gtk::prelude::*;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmIterChildrenExt,
};

use super::{actions::header::ActionsHeader, flows::header::FlowsHeader};

#[derive(Debug)]
pub enum HeaderBarInput {
    ChangedView(String),
}

#[derive(Debug)]
pub struct HeaderBarModel {
    action_header_rc: Rc<Controller<ActionsHeader>>,
    flow_header_rc: Rc<Controller<FlowsHeader>>,
}

impl HeaderBarModel {
    fn swap_content(&mut self, swap_target: &gtk::Box, new_content: &gtk::Box) {
        for child in swap_target.iter_children() {
            swap_target.remove(&child);
        }
        swap_target.append(new_content);
    }
}

#[relm4::component(pub)]
impl Component for HeaderBarModel {
    type Init = (
        Rc<Controller<ActionsHeader>>,
        Rc<Controller<FlowsHeader>>,
        Rc<adw::ViewStack>,
    );
    type Input = HeaderBarInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::HeaderBar {
            #[name = "start_box"]
            pack_start = &gtk::Box,

            #[wrap(Some)]
            set_title_widget = &adw::ViewSwitcher {
                #[local_ref]
                #[wrap(Some)]
                set_stack = stack -> adw::ViewStack,
            },

            #[name = "end_box"]
            pack_end = &gtk::Box,
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderBarModel {
            action_header_rc: init.0,
            flow_header_rc: init.1,
        };

        let stack = &*init.2;
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
            HeaderBarInput::ChangedView(new_view) => {
                if new_view == "flows" {
                    let rc_clone = self.flow_header_rc.clone();
                    self.swap_content(&widgets.start_box, &rc_clone.widgets().start);
                    self.swap_content(&widgets.end_box, &rc_clone.widgets().end);
                } else if new_view == "actions" {
                    let rc_clone = self.action_header_rc.clone();
                    self.swap_content(&widgets.start_box, &rc_clone.widgets().start);
                    self.swap_content(&widgets.end_box, &rc_clone.widgets().end);
                } else {
                    self.swap_content(&widgets.start_box, &gtk::Box::builder().build());
                    self.swap_content(&widgets.end_box, &gtk::Box::builder().build());
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
