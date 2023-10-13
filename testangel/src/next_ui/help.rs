use gtk::prelude::*;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use rust_i18n::t;

#[derive(Debug)]
pub struct HelpModel {
    support_location: String,
}

#[relm4::component(pub)]
impl SimpleComponent for HelpModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    set_markup: &format!(r#"<span size="xx-large">{}</span> v{}"#, t!("name"), env!("CARGO_PKG_VERSION")),
                },
                gtk::Label {
                    set_label: &model.support_location,
                },
            },
            gtk::Notebook {
                set_tab_pos: gtk::PositionType::Left,
                set_vexpand: true,

                // Getting Started
                append_page[Some(&gtk::Label::new(Some(&t!("help.notebook.getting-started.header"))))] = &gtk::Label {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_justify: gtk::Justification::Left,
                    set_wrap: true,
                    set_selectable: true,
                    set_margin_all: 5,

                    set_markup: &t!("help.notebook.getting-started.content"),
                },

                // Flows
                append_page[Some(&gtk::Label::new(Some(&t!("help.notebook.flows.header"))))] = &gtk::Label {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_justify: gtk::Justification::Left,
                    set_wrap: true,
                    set_selectable: true,
                    set_margin_all: 5,

                    set_markup: &t!("help.notebook.flows.content"),
                },

                // Actions
                append_page[Some(&gtk::Label::new(Some(&t!("help.notebook.actions.header"))))] = &gtk::Label {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_justify: gtk::Justification::Left,
                    set_wrap: true,
                    set_selectable: true,
                    set_margin_all: 5,

                    set_markup: &t!("help.notebook.actions.content"),
                },

                // Engines
                append_page[Some(&gtk::Label::new(Some(&t!("help.notebook.engines.header"))))] = &gtk::Label {
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,
                    set_justify: gtk::Justification::Left,
                    set_wrap: true,
                    set_selectable: true,
                    set_margin_all: 5,

                    set_markup: &t!("help.notebook.engines.content"),
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HelpModel {
            support_location: if let Ok(local_support) = std::env::var("TA_LOCAL_SUPPORT_CONTACT") {
                t!("help.contact-local", contact = local_support)
            } else {
                t!("help.repository")
            },
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
