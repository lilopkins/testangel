use std::rc::Rc;

use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentParts, ComponentSender, Controller, RelmWidgetExt, SimpleComponent,
};
use testangel::types::AutomationFlow;
use rust_i18n::t;

mod action_component;

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
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsHeader {
    type Init = ();
    type Input = ();
    type Output = FlowsHeaderOutput;

    view! {
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_icon_name: relm4_icons::icon_name::PAPER,
                set_tooltip: &t!("flows.header.new"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::NewFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::LOUPE,
                set_tooltip: &t!("flows.header.open"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::OpenFlow).unwrap();
                },
            },
            gtk::Button {
                set_icon_name: relm4_icons::icon_name::FLOPPY,
                set_tooltip: &t!("flows.header.save"),
                connect_clicked[sender] => move |_| {
                    // unwrap rationale: receivers will never be dropped
                    sender.output(FlowsHeaderOutput::SaveFlow).unwrap();
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
            gtk::MenuButton {
                set_icon_name: relm4_icons::icon_name::MENU,
                set_tooltip: &t!("flows.header.more"),
                set_direction: gtk::ArrowType::Down,

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_position: gtk::PositionType::Bottom,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 1,

                        gtk::Button {
                            set_label: "Save flow as...",
                            add_css_class: "flat",

                            connect_clicked[sender] => move |_| {
                                // unwrap rationale: receivers will never be dropped
                                sender.output(FlowsHeaderOutput::SaveAsFlow).unwrap();
                            },
                        },

                        gtk::Button {
                            set_label: "Close flow",
                            add_css_class: "flat",

                            connect_clicked[sender] => move |_| {
                                // unwrap rationale: receivers will never be dropped
                                sender.output(FlowsHeaderOutput::CloseFlow).unwrap();
                            },
                        },
                    },
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsHeader;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

#[derive(Debug)]
pub struct FlowsModel {
    open_flow: Option<AutomationFlow>,
    header: Rc<Controller<FlowsHeader>>,
}

impl FlowsModel {
    /// Get an [`Rc`] clone of the header controller
    pub fn header_controller_rc(&self) -> Rc<Controller<FlowsHeader>> {
        self.header.clone()
    }
}

#[relm4::component(pub)]
impl SimpleComponent for FlowsModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,

            gtk::Label {
                set_markup: r#"<span size="large">Flows</span>"#,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Rc::new(FlowsHeader::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                _ => (),
            }));

        let model = FlowsModel {
            open_flow: None,
            header,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
