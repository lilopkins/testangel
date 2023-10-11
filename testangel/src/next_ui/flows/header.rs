use adw::prelude::*;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
    SimpleComponent,
};
use rust_i18n::t;

#[derive(Debug)]
pub struct FlowsHeader;

#[derive(Debug)]
pub enum FlowsHeaderOutput {
    NewFlow,
    OpenFlow,
    SaveFlow,
    SaveAsFlow,
    CloseFlow,
    RunFlow,
    AddStep,
}

#[derive(Debug)]
pub enum FlowsHeaderInput {
    OpenAboutDialog,
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsHeader {
    type Init = ();
    type Input = FlowsHeaderInput;
    type Output = FlowsHeaderOutput;

    view! {
        #[root]
        #[name = "start"]
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_label: &t!("open"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::OpenFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::PLUS,
                set_tooltip: &t!("flows.header.add"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::AddStep).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::PLAY,
                set_tooltip: &t!("flows.header.run"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::RunFlow).unwrap();
                },
            },
        },

        #[name = "end"]
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_label: &t!("save"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::SaveFlow).unwrap();
                },
            },
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &t!("flows.header.more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&flows_menu)) {
                    set_position: gtk::PositionType::Bottom,
                },
            },
        },

        menu_save_widget = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            #[name = "group"]
            gtk::ToggleButton {
                set_label: "O",
            },
            gtk::ToggleButton {
                set_label: "S",
                set_group: Some(&group),
            },
            gtk::ToggleButton {
                set_label: "SA",
                set_group: Some(&group),
            },
        },
    }

    menu! {
        flows_menu: {
            custom: "menu_save_widget",
            &t!("flows.header.new") => FlowsNewAction,
            &t!("flows.header.save-as") => FlowsSaveAsAction,
            &t!("flows.header.close") => FlowsCloseAction,
            section! {
                &t!("header.about") => FlowsAboutAction,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader;
        let widgets = view_output!();

        let about_action: RelmAction<FlowsAboutAction> = RelmAction::new_stateless(move |_| {
            sender.input(FlowsHeaderInput::OpenAboutDialog);
        });
        relm4::main_application().set_accelerators_for_action::<FlowsAboutAction>(&["<primary>A"]);

        let mut group = RelmActionGroup::<FlowsActionGroup>::new();
        group.add_action(about_action);
        group.register_for_widget(&widgets.end);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FlowsHeaderInput::OpenAboutDialog => {
                crate::next_ui::about::AppAbout::builder()
                    // TODO .transient_for()
                    .launch(())
                    .widget()
                    .show();
            }
        }
    }
}

relm4::new_action_group!(FlowsActionGroup, "flows");
relm4::new_stateless_action!(FlowsNewAction, FlowsActionGroup, "new");
relm4::new_stateless_action!(FlowsSaveAsAction, FlowsActionGroup, "save-as");
relm4::new_stateless_action!(FlowsCloseAction, FlowsActionGroup, "close");
relm4::new_stateless_action!(FlowsAboutAction, FlowsActionGroup, "about");
