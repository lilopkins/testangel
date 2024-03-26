use adw::prelude::*;
use relm4::{adw, gtk, Component};
use testangel::types::Action;

use crate::ui::lang;

#[derive(Debug)]
pub enum MetadataInput {
    /// Inform the metadata component that the action has changed and as such
    /// it should reload the metadata values
    ChangeAction(Action),
}

#[derive(Clone, Debug, Default)]
pub struct MetadataOutput {
    pub new_name: Option<String>,
    pub new_group: Option<String>,
    pub new_author: Option<String>,
    pub new_description: Option<String>,
    pub new_visible: Option<bool>,
}

#[derive(Debug)]
pub struct Metadata;

#[relm4::component(pub)]
impl Component for Metadata {
    type Init = ();
    type Input = MetadataInput;
    type Output = MetadataOutput;
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            set_title: &lang::lookup("action-metadata-label"),

            #[name = "name"]
            adw::EntryRow {
                set_title: &lang::lookup("action-metadata-name"),

                connect_changed[sender] => move |entry| {
                    let _ = sender.output(MetadataOutput {
                        new_name: Some(entry.text().to_string()),
                        ..Default::default()
                    });
                },
            },
            #[name = "group"]
            adw::EntryRow {
                set_title: &lang::lookup("action-metadata-group"),

                connect_changed[sender] => move |entry| {
                    let _ = sender.output(MetadataOutput {
                        new_group: Some(entry.text().to_string()),
                        ..Default::default()
                    });
                },
            },
            #[name = "author"]
            adw::EntryRow {
                set_title: &lang::lookup("action-metadata-author"),

                connect_changed[sender] => move |entry| {
                    let _ = sender.output(MetadataOutput {
                        new_author: Some(entry.text().to_string()),
                        ..Default::default()
                    });
                },
            },
            #[name = "description"]
            adw::EntryRow {
                set_title: &lang::lookup("action-metadata-description"),

                connect_changed[sender] => move |entry| {
                    let _ = sender.output(MetadataOutput {
                        new_description: Some(entry.text().to_string()),
                        ..Default::default()
                    });
                },
            },
            adw::ActionRow {
                set_title: &lang::lookup("action-metadata-visible"),

                #[name = "visible"]
                add_suffix = &gtk::Switch {
                    set_margin_top: 12,
                    set_margin_bottom: 12,

                    connect_state_set[sender] => move |_switch, state| {
                        let _ = sender.output(MetadataOutput {
                            new_visible: Some(state),
                            ..Default::default()
                        });
                        gtk::glib::signal::Propagation::Stop
                    },
                },

            }
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Metadata;
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MetadataInput::ChangeAction(action) => {
                widgets.name.set_text(&action.friendly_name);
                widgets.group.set_text(&action.group);
                widgets.author.set_text(&action.author);
                widgets.description.set_text(&action.description);
                widgets.visible.set_active(action.visible);
            }
        }

        self.update_view(widgets, sender)
    }
}
