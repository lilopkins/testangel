use std::{collections::HashMap, sync::Arc};

use relm4::{adw, gtk, Component, ComponentParts, RelmWidgetExt};
use adw::prelude::*;
use rust_i18n::t;
use testangel::{types::{AutomationFlow, FlowError}, ipc::EngineList, action_loader::ActionMap, report_generation::{self, ReportGenerationError}};
use testangel_ipc::prelude::{Evidence, ParameterValue, EvidenceContent};

#[derive(Debug)]
pub enum ExecutionDialogCommandOutput {
    /// Execution completed with the resulting evidence
    Complete(Vec<Evidence>),

    /// Execution failed at the given step and for the given reason
    Failed(usize, FlowError),
}

#[derive(Debug)]
pub struct ExecutionDialogInit {
    pub flow: AutomationFlow,
    pub engine_list: Arc<EngineList>,
    pub action_map: Arc<ActionMap>,
}

#[derive(Debug)]
pub enum ExecutionDialogInput {
    Close,
    FailedToGenerateReport(ReportGenerationError),
}

#[derive(Debug)]
pub struct ExecutionDialog;

impl ExecutionDialog {
    /// Create the absolute barebones of a message dialog, allowing for custom button and response mapping.
    fn create_message_dialog<S>(&self, title: S, message: S) -> adw::MessageDialog
    where
        S: AsRef<str>,
    {
        adw::MessageDialog::builder()
            .title(title.as_ref())
            .heading(title.as_ref())
            .body(message.as_ref())
            .modal(true)
            .build()
    }
}

#[relm4::component(pub)]
impl Component for ExecutionDialog {
    type Init = ExecutionDialogInit;
    type Input = ExecutionDialogInput;
    type Output = ();
    type CommandOutput = ExecutionDialogCommandOutput;

    view! {
        #[root]
        adw::Window {
            set_modal: true,
            set_resizable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 50,

                gtk::Spinner {
                    set_spinning: true,
                },
                gtk::Label {
                    set_label: &t!("flows.execution.running"),
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ExecutionDialog;
        let widgets = view_output!();
        let flow = init.flow;
        let engine_list = init.engine_list.clone();
        let action_map = init.action_map.clone();

        sender.spawn_oneshot_command(move || {
            let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
            let mut evidence = Vec::new();

            for engine in engine_list.inner() {
                if engine.reset_state().is_err() {
                    evidence.push(Evidence {
                        label: String::from("WARNING: State Warning"),
                        content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
                    });
                }
            }

            for (step, action_config) in flow.actions.iter().enumerate() {
                match action_config.execute(
                    action_map.clone(),
                    engine_list.clone(),
                    outputs.clone(),
                ) {
                    Ok((output, ev)) => {
                        outputs.push(output);
                        evidence = [evidence, ev].concat();
                    }
                    Err(e) => {
                        return ExecutionDialogCommandOutput::Failed(step + 1, e);
                    }
                }
            }

            ExecutionDialogCommandOutput::Complete(evidence)
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>, root: &Self::Root) {
        match message {
            ExecutionDialogInput::Close => root.destroy(),
            ExecutionDialogInput::FailedToGenerateReport(reason) => {
                let dialog = self.create_message_dialog(t!("flows.execution.report-failed"), t!("flows.exection.report-failed-message", reason = reason.to_string()));
                dialog.set_transient_for(Some(root));
                dialog.add_response("ok", &t!("ok"));
                dialog.set_default_response(Some("ok"));
                let sender_c = sender.clone();
                dialog.connect_response(None, move |dlg, _response| {
                    sender_c.input(ExecutionDialogInput::Close);
                    dlg.close();
                });
                dialog.show();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            ExecutionDialogCommandOutput::Complete(evidence) => {
                log::info!("Execution complete.");

                // Present save dialog
                let filter = gtk::FileFilter::new();
                filter.set_name(Some(&t!("flows.execution.pdf-files")));
                filter.add_suffix("pdf");
                filter.add_mime_type("application/pdf");

                let dialog = gtk::FileChooserDialog::builder()
                    .transient_for(root)
                    .modal(true)
                    .action(gtk::FileChooserAction::Save)
                    .create_folders(true)
                    .title(t!("flows.execution.save-title"))
                    .filter(&filter)
                    .build();

                dialog.add_button(&t!("save"), gtk::ResponseType::Ok);

                let sender_c = sender.clone();
                dialog.connect_response(move |dialog, response| {
                    if response == gtk::ResponseType::Ok {
                        dialog.close();
                        if let Some(path) = dialog.file() {
                            let path = path.path().unwrap();
                            if let Err(e) =
                                report_generation::save_report(path.with_extension("pdf"), evidence.clone())
                            {
                                // Failed to generate report
                                sender_c.input(ExecutionDialogInput::FailedToGenerateReport(e));
                                return;
                            } else if let Err(e) = opener::open(path.with_extension("pdf")) {
                                log::warn!("Failed to open evidence: {e}");
                            }
                        }
                        sender_c.input(ExecutionDialogInput::Close);
                    }
                });
                dialog.show();
            }

            ExecutionDialogCommandOutput::Failed(step, reason) => {
                log::warn!("Execution failed");
                let dialog = self.create_message_dialog(t!("flows.execution.failed"), t!("flows.exection.failed-message", step = step, reason = reason.to_string()));
                dialog.set_transient_for(Some(root));
                dialog.add_response("ok", &t!("ok"));
                dialog.set_default_response(Some("ok"));
                let sender_c = sender.clone();
                dialog.connect_response(None, move |dlg, _response| {
                    sender_c.input(ExecutionDialogInput::Close);
                    dlg.close();
                });
                dialog.show();
            }
        }
    }
}