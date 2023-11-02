use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmIterChildrenExt, RelmWidgetExt,
};
use testangel::{action_loader::ActionMap, ipc::EngineList};

use crate::ui::lang;

use super::{actions::header::{ActionsHeader, ActionsHeaderInput}, flows::header::{FlowsHeader, FlowsHeaderInput}};

#[derive(Debug)]
pub enum HeaderBarInput {
    ChangedView(String),
    OpenAboutDialog,
    ActionsMapChanged(Arc<ActionMap>),
    NewFile,
    OpenFile,
    SaveFile,
    SaveAsFile,
    CloseFile,
}

#[derive(Debug)]
pub enum HeaderBarOutput {
    AttachFileActionGroup(RelmActionGroup<FileActionGroup>),
    AttachGeneralActionGroup(RelmActionGroup<GeneralActionGroup>),
}

#[derive(Debug)]
enum MenuTarget {
    Nothing,
    Flows,
    Actions,
}

#[derive(Debug)]
pub struct HeaderBarModel {
    currently_menu_target: MenuTarget,
    engine_list: Arc<EngineList>,
    action_map: Arc<ActionMap>,
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

            pack_end = &gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &lang::lookup("header-more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&menu)) {
                    set_position: gtk::PositionType::Bottom,
                },
            },
        }
    }

    menu! {
        menu: {
            &lang::lookup("header-new") => FileNewAction,
            &lang::lookup("header-open") => FileOpenAction,
            &lang::lookup("header-save") => FileSaveAction,
            &lang::lookup("header-save-as") => FileSaveAsAction,
            &lang::lookup("header-close") => FileCloseAction,
            section! {
                &lang::lookup("header-about") => GeneralAboutAction,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderBarModel {
            currently_menu_target: MenuTarget::Nothing,
            action_header_rc: init.0,
            flow_header_rc: init.1,
            engine_list: init.3,
            action_map: init.4,
        };

        let stack = &*init.2;
        let widgets = view_output!();

        let sender_c = sender.clone();
        let new_action: RelmAction<FileNewAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.input(HeaderBarInput::NewFile);
        });
        relm4::main_application().set_accelerators_for_action::<FileNewAction>(&["<primary>N"]);

        let sender_c = sender.clone();
        let open_action: RelmAction<FileOpenAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.input(HeaderBarInput::OpenFile);
        });
        relm4::main_application().set_accelerators_for_action::<FileOpenAction>(&["<primary>O"]);

        let sender_c = sender.clone();
        let save_action: RelmAction<FileSaveAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.input(HeaderBarInput::SaveFile);
        });
        relm4::main_application().set_accelerators_for_action::<FileSaveAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let save_as_action: RelmAction<FileSaveAsAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.input(HeaderBarInput::SaveAsFile);
        });
        relm4::main_application()
            .set_accelerators_for_action::<FileSaveAsAction>(&["<primary>S"]);

        let sender_c = sender.clone();
        let close_action: RelmAction<FileCloseAction> = RelmAction::new_stateless(move |_| {
            // unwrap rationale: receiver will never be disconnected
            sender_c.input(HeaderBarInput::CloseFile);
        });
        relm4::main_application()
            .set_accelerators_for_action::<FileCloseAction>(&["<primary>W"]);

        let sender_c = sender.clone();
        let about_action: RelmAction<GeneralAboutAction> = RelmAction::new_stateless(move |_| {
            sender_c.input(HeaderBarInput::OpenAboutDialog);
        });
        relm4::main_application()
            .set_accelerators_for_action::<GeneralAboutAction>(&["F1"]);

        let mut group = RelmActionGroup::<FileActionGroup>::new();
        group.add_action(new_action);
        group.add_action(open_action);
        group.add_action(save_action);
        group.add_action(save_as_action);
        group.add_action(close_action);
        let _ = sender.output(HeaderBarOutput::AttachFileActionGroup(group));

        let mut group = RelmActionGroup::<GeneralActionGroup>::new();
        group.add_action(about_action);
        let _ = sender.output(HeaderBarOutput::AttachGeneralActionGroup(group));

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
                    self.currently_menu_target = MenuTarget::Flows;
                } else if new_view == "actions" {
                    let rc_clone = self.action_header_rc.clone();
                    self.swap_content(&widgets.start_box, &rc_clone.widgets().start);
                    self.currently_menu_target = MenuTarget::Actions;
                } else {
                    self.swap_content(&widgets.start_box, &gtk::Box::builder().build());
                    self.currently_menu_target = MenuTarget::Nothing;
                }
            }
            HeaderBarInput::NewFile => {
                match self.currently_menu_target {
                    MenuTarget::Nothing => (),
                    MenuTarget::Flows => {
                        self.flow_header_rc.emit(FlowsHeaderInput::PleaseOutput(super::flows::header::FlowsHeaderOutput::NewFlow));
                    }
                    MenuTarget::Actions => {
                        self.action_header_rc.emit(ActionsHeaderInput::PleaseOutput(super::actions::header::ActionsHeaderOutput::NewAction));
                    }
                }
            }
            HeaderBarInput::OpenFile => {
                match self.currently_menu_target {
                    MenuTarget::Nothing => (),
                    MenuTarget::Flows => {
                        self.flow_header_rc.emit(FlowsHeaderInput::PleaseOutput(super::flows::header::FlowsHeaderOutput::OpenFlow));
                    }
                    MenuTarget::Actions => {
                        self.action_header_rc.emit(ActionsHeaderInput::PleaseOutput(super::actions::header::ActionsHeaderOutput::OpenAction));
                    }
                }
            }
            HeaderBarInput::SaveFile => {
                match self.currently_menu_target {
                    MenuTarget::Nothing => (),
                    MenuTarget::Flows => {
                        self.flow_header_rc.emit(FlowsHeaderInput::PleaseOutput(super::flows::header::FlowsHeaderOutput::SaveFlow));
                    }
                    MenuTarget::Actions => {
                        self.action_header_rc.emit(ActionsHeaderInput::PleaseOutput(super::actions::header::ActionsHeaderOutput::SaveAction));
                    }
                }
            }
            HeaderBarInput::SaveAsFile => {
                match self.currently_menu_target {
                    MenuTarget::Nothing => (),
                    MenuTarget::Flows => {
                        self.flow_header_rc.emit(FlowsHeaderInput::PleaseOutput(super::flows::header::FlowsHeaderOutput::SaveAsFlow));
                    }
                    MenuTarget::Actions => {
                        self.action_header_rc.emit(ActionsHeaderInput::PleaseOutput(super::actions::header::ActionsHeaderOutput::SaveAsAction));
                    }
                }
            }
            HeaderBarInput::CloseFile => {
                match self.currently_menu_target {
                    MenuTarget::Nothing => (),
                    MenuTarget::Flows => {
                        self.flow_header_rc.emit(FlowsHeaderInput::PleaseOutput(super::flows::header::FlowsHeaderOutput::CloseFlow));
                    }
                    MenuTarget::Actions => {
                        self.action_header_rc.emit(ActionsHeaderInput::PleaseOutput(super::actions::header::ActionsHeaderOutput::CloseAction));
                    }
                }
            }
        }
        self.update_view(widgets, sender);
    }
}

relm4::new_action_group!(pub FileActionGroup, "file");
relm4::new_stateless_action!(FileNewAction, FileActionGroup, "new");
relm4::new_stateless_action!(FileOpenAction, FileActionGroup, "open");
relm4::new_stateless_action!(FileSaveAction, FileActionGroup, "save");
relm4::new_stateless_action!(FileSaveAsAction, FileActionGroup, "save-as");
relm4::new_stateless_action!(FileCloseAction, FileActionGroup, "close");

impl std::fmt::Debug for FileActionGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileActionGroup")
    }
}

relm4::new_action_group!(pub GeneralActionGroup, "general");
relm4::new_stateless_action!(pub GeneralAboutAction, GeneralActionGroup, "about");

impl std::fmt::Debug for GeneralActionGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeneralActionGroup")
    }
}
