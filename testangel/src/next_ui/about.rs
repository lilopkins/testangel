use relm4::{adw, gtk, SimpleComponent};
use rust_i18n::t;

pub struct AppAbout;

#[relm4::component(pub)]
impl SimpleComponent for AppAbout {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        #[name = "about"]
        adw::AboutWindow {
            set_application_name: &t!("name"),
            set_version: env!("CARGO_PKG_VERSION"),
            set_issue_url: &support_url,
            set_developer_name: "Lily Hopkins",
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppAbout;

        let support_url = std::env::var("TA_LOCAL_SUPPORT_CONTACT")
            .unwrap_or("https://github.com/lilopkins/testangel".to_string());
        let widgets = view_output!();
        widgets.about.add_acknowledgement_section(
            Some(&t!("about.testing")),
            &["John Chander", "Eden Turner"],
        );
        widgets
            .about
            .add_legal_section("TestAngel", None, gtk::License::Gpl30Only, None);
        widgets
            .about
            .add_legal_section("GTK", None, gtk::License::Gpl20Only, None);
        widgets
            .about
            .add_legal_section("Adwaita", None, gtk::License::Gpl20Only, None);
        widgets
            .about
            .add_legal_section("clap", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("fern", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("libloading", None, gtk::License::Custom, Some("ISC"));
        widgets
            .about
            .add_legal_section("log", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("image", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("thiserror", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("pretty_env_logger", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("serde", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("uuid", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("ron", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("genpdf", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("chrono", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("base64", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("itertools", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("octocrab", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("semver", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("relm4", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("relm4-icons", None, gtk::License::MitX11, None);
        widgets
            .about
            .add_legal_section("rust-i18n", None, gtk::License::MitX11, None);

        relm4::ComponentParts { model, widgets }
    }
}
