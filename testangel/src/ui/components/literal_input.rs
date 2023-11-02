use adw::prelude::*;
use relm4::{adw, gtk, SimpleComponent};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

use crate::ui::lang;

#[allow(dead_code)]
#[derive(Debug)]
pub struct LiteralInput {
    kind: ParameterKind,
}

#[derive(Debug)]
pub enum LiteralInputOutput {
    /// The value stored within this literal input has changed
    ValueChanged(ParameterValue),
}

#[derive(Debug)]
pub enum LiteralInputWidgets {
    String { entry: gtk::Entry },
    Integer { entry: gtk::SpinButton },
    Decimal { entry: gtk::SpinButton },
    Boolean { entry: gtk::CheckButton },
}

impl SimpleComponent for LiteralInput {
    type Input = ();
    type Output = LiteralInputOutput;
    type Init = ParameterValue;
    type Root = adw::Bin;
    type Widgets = LiteralInputWidgets;

    fn init_root() -> Self::Root {
        adw::Bin::default()
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self { kind: init.kind() };

        let widgets = match &init {
            ParameterValue::String(val) => {
                let entry = gtk::Entry::builder()
                    .text(val)
                    .placeholder_text(lang::lookup("value"))
                    .build();
                let sender_c = sender.clone();
                entry.connect_changed(move |etry| {
                    let _ = sender_c.clone().output(LiteralInputOutput::ValueChanged(
                        ParameterValue::String(etry.text().to_string()),
                    ));
                });
                root.set_child(Some(&entry));
                LiteralInputWidgets::String { entry }
            }
            ParameterValue::Integer(val) => {
                let entry = gtk::SpinButton::builder()
                    .adjustment(&gtk::Adjustment::new(
                        *val as f64,
                        i32::MIN as f64,
                        i32::MAX as f64,
                        1.,
                        5.,
                        0.,
                    ))
                    .digits(0)
                    .numeric(true)
                    .editable(true)
                    .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
                    .build();
                entry.set_increments(1., 10.);
                let sender_c = sender.clone();
                entry.connect_value_changed(move |spn| {
                    let _ = sender_c.clone().output(LiteralInputOutput::ValueChanged(
                        ParameterValue::Integer(spn.value() as i32),
                    ));
                });
                root.set_child(Some(&entry));
                LiteralInputWidgets::Integer { entry }
            }
            ParameterValue::Decimal(val) => {
                let entry = gtk::SpinButton::builder()
                    .adjustment(&gtk::Adjustment::new(
                        *val as f64,
                        f32::MIN as f64,
                        f32::MAX as f64,
                        0.1,
                        1.,
                        0.,
                    ))
                    .digits(2)
                    .numeric(true)
                    .editable(true)
                    .update_policy(gtk::SpinButtonUpdatePolicy::IfValid)
                    .build();
                let sender_c = sender.clone();
                entry.connect_value_changed(move |spn| {
                    let _ = sender_c.clone().output(LiteralInputOutput::ValueChanged(
                        ParameterValue::Decimal(spn.value() as f32),
                    ));
                });
                root.set_child(Some(&entry));
                LiteralInputWidgets::Decimal { entry }
            }
            ParameterValue::Boolean(val) => {
                let entry = gtk::CheckButton::builder()
                    .active(*val)
                    .label(lang::lookup("value"))
                    .build();
                let sender_c = sender.clone();
                entry.connect_toggled(move |chk| {
                    let _ = sender_c.clone().output(LiteralInputOutput::ValueChanged(
                        ParameterValue::Boolean(chk.is_active()),
                    ));
                });
                root.set_child(Some(&entry));
                LiteralInputWidgets::Boolean { entry }
            }
        };

        relm4::ComponentParts { model, widgets }
    }
}
