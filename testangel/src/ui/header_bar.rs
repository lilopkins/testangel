use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmIterChildrenExt,
};
use testangel::{action_loader::ActionMap, ipc::EngineList};

use super::{actions::header::ActionsHeader, flows::header::FlowsHeader};

#[derive(Debug)]
pub enum HeaderBarInput {
    ChangedView(String),
    OpenAboutDialog,
    ActionsMapChanged(Arc<ActionMap>),
}

#[derive(Debug)]
pub enum HeaderBarOutput {
    AttachActionGroup(RelmActionGroup<GeneralActionGroup>),
}

#[derive(Debug)]
pub struct HeaderBarModel {
    engine_list: Arc<EngineList>,
    action_map: Arc<ActionMap>,
    action_header_rc: Rc<Controller<ActionsHeader>>,
    flow_header_rc: Rc<Controller<FlowsHeader>>,
}

impl std::fmt::Debug for GeneralActionGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeneralActionGroup")
    }
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
        Arc<EngineList>,
        Arc<ActionMap>,
    );
    type Input = HeaderBarInput;
    type Output = HeaderBarOutput;
    type CommandOutput = ();

    view! {
        #[root]
        root = adw::HeaderBar {
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
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderBarModel {
            action_header_rc: init.0,
            flow_header_rc: init.1,
            engine_list: init.3,
            action_map: init.4,
        };

        let stack = &*init.2;
        let widgets = view_output!();

        let sender_c = sender.clone();
        let about_action: RelmAction<GeneralAboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(HeaderBarInput::OpenAboutDialog);
        });
        relm4::main_application()
            .set_accelerators_for_action::<GeneralAboutAction>(&["<primary>A"]);
        let mut group = RelmActionGroup::<GeneralActionGroup>::new();
        group.add_action(about_action);
        let _ = sender.output(HeaderBarOutput::AttachActionGroup(group));

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
            HeaderBarInput::ActionsMapChanged(new_map) => self.action_map = new_map,
            HeaderBarInput::OpenAboutDialog => {
                crate::ui::about::AppAbout::builder()
                    .transient_for(root)
                    .launch((self.engine_list.clone(), self.action_map.clone()))
                    .widget()
                    .set_visible(true);
            }
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

relm4::new_action_group!(pub GeneralActionGroup, "general");
relm4::new_stateless_action!(pub GeneralAboutAction, GeneralActionGroup, "about");
