use gtk::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
pub struct ActionsModel;

#[relm4::component(pub)]
impl SimpleComponent for ActionsModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,

            adw::StatusPage {
                set_title: "Not yet implemented",
                set_description: Some("Actions are not implemented in this UI preview for TestAngel"),
                set_icon_name: Some(relm4_icons::icon_name::HOURGLASS),
                set_vexpand: true,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ActionsModel;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
