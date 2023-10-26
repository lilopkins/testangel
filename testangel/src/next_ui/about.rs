use std::sync::Arc;

use relm4::{adw, gtk, SimpleComponent};
use testangel::{action_loader::ActionMap, ipc::EngineList};

use crate::next_ui::lang;

pub struct AppAbout;

#[relm4::component(pub)]
impl SimpleComponent for AppAbout {
    type Init = (Arc<EngineList>, Arc<ActionMap>);
    type Input = ();
    type Output = ();

    view! {
        #[root]
        #[name = "about"]
        adw::AboutWindow {
            set_application_icon: "testangel",
            set_application_name: &lang::lookup("app-name"),
            set_version: env!("CARGO_PKG_VERSION"),
            set_copyright: "© 2023 Lily Hopkins",
            set_license_type: gtk::License::Gpl30Only,
            set_issue_url: &support_url,
            set_developer_name: "Lily Hopkins",
            set_debug_info: &log_data,

            add_acknowledgement_section: (Some(&lang::lookup("acknowledgements-testing-title")), &["John Chander", "Eden Turner"]),
            add_acknowledgement_section: (Some(&lang::lookup("acknowledgements-translations-title")), &["Lily Hopkins"]),
            add_legal_section: ("GTK",               None, gtk::License::Gpl20Only, None),
            add_legal_section: ("Adwaita",           None, gtk::License::Gpl20Only, None),
            add_legal_section: ("clap",              None, gtk::License::MitX11,    None),
            add_legal_section: ("fern",              None, gtk::License::MitX11,    None),
            add_legal_section: ("libloading",        None, gtk::License::Custom,    Some("ISC")),
            add_legal_section: ("log",               None, gtk::License::MitX11,    None),
            add_legal_section: ("image",             None, gtk::License::MitX11,    None),
            add_legal_section: ("thiserror",         None, gtk::License::MitX11,    None),
            add_legal_section: ("pretty_env_logger", None, gtk::License::MitX11,    None),
            add_legal_section: ("serde",             None, gtk::License::MitX11,    None),
            add_legal_section: ("uuid",              None, gtk::License::MitX11,    None),
            add_legal_section: ("ron",               None, gtk::License::MitX11,    None),
            add_legal_section: ("genpdf",            None, gtk::License::MitX11,    None),
            add_legal_section: ("chrono",            None, gtk::License::MitX11,    None),
            add_legal_section: ("base64",            None, gtk::License::MitX11,    None),
            add_legal_section: ("itertools",         None, gtk::License::MitX11,    None),
            add_legal_section: ("octocrab",          None, gtk::License::MitX11,    None),
            add_legal_section: ("semver",            None, gtk::License::MitX11,    None),
            add_legal_section: ("relm4",             None, gtk::License::MitX11,    None),
            add_legal_section: ("relm4-icons",       None, gtk::License::MitX11,    None),
            add_legal_section: ("rust-i18n",         None, gtk::License::MitX11,    None),
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppAbout;

        let engine_list = init.0;
        let action_map = init.1;
        let log_data = format!(
            "Debug data generated at: {}\nSoftware version: {}\nLocale: {} (system wanted: {:?})\n\nEngines:\n{:#?}\n\nActions:\n{:#?}",
            chrono::Local::now(),
            env!("CARGO_PKG_VERSION"),
            lang::current_locale(),
            sys_locale::get_locales().collect::<Vec<_>>(),
            engine_list.inner(),
            action_map.get_by_group(),
        );
        let support_url = std::env::var("TA_LOCAL_SUPPORT_CONTACT")
            .unwrap_or("https://github.com/lilopkins/testangel".to_string());
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }
}
