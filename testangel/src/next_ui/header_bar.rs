use std::rc::Rc;

use gtk::prelude::*;
use relm4::{
    adw, gtk, ComponentParts, ComponentSender, Controller, RelmIterChildrenExt, SimpleComponent, ComponentController,
};

use super::flows::FlowsHeader;

#[derive(Debug)]
pub enum HeaderBarInput {
    ChangedView(String),
}

#[derive(Debug)]
pub struct HeaderBarModel {
    start_box: gtk::Box,
    flow_header_rc: Rc<Controller<FlowsHeader>>,
}

impl HeaderBarModel {
    fn change_start_box(&mut self, new_box: &gtk::Box) {
        for child in self.start_box.iter_children() {
            self.start_box.remove(&child);
        }
        self.start_box.append(new_box);
    }
}

#[relm4::component(pub)]
impl SimpleComponent for HeaderBarModel {
    type Init = (Rc<Controller<FlowsHeader>>, Rc<adw::ViewStack>);
    type Input = HeaderBarInput;
    type Output = ();

    view! {
        #[root]
        adw::HeaderBar {
            #[local_ref]
            pack_start = start_box -> gtk::Box,

            #[wrap(Some)]
            set_title_widget = &adw::ViewSwitcher {
                #[local_ref]
                #[wrap(Some)]
                set_stack = stack -> adw::ViewStack,

                connect_stack_notify => |_| {
                    log::debug!("SIGNAL!");
                },
            },

            pack_end = &gtk::Box {

            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderBarModel {
            flow_header_rc: init.0,
            start_box: gtk::Box::new(gtk::Orientation::Horizontal, 0),
        };

        let stack = &*init.1;
        let start_box = &model.start_box;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            HeaderBarInput::ChangedView(new_view) => {
                if new_view == "flows" {
                    let rc_clone = self.flow_header_rc.clone();
                    self.change_start_box(rc_clone.widget());
                } else {
                    self.change_start_box(&gtk::Box::builder().build());
                }
            }
        }
    }
}
