use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
pub struct FlowsModel;

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
                set_label: r#"<span size="large">Flows</span>"#,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = FlowsModel;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
