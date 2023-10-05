use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

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

            gtk::Label {
                set_markup: r#"<span size="large">Actions</span>"#,
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
