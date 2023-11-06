use std::{collections::HashMap, sync::Arc};

use adw::prelude::*;
use relm4::{adw, gtk, Component, ComponentParts, RelmWidgetExt};
use testangel::{
    action_loader::ActionMap,
    ipc::EngineList,
    report_generation::{self, ReportGenerationError},
    types::{AutomationFlow, FlowError},
};
use testangel_ipc::prelude::{Evidence, EvidenceContent, ParameterValue};

use crate::ui::{file_filters, lang};

#[derive(Debug)]
pub enum ExecutionDialogCommandOutput {
    /// Execution completed with the resulting evidence
    Complete(Vec<Evidence>),

    /// Execution failed at the given step and for the given reason
    Failed(usize, FlowError, Vec<Evidence>),
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
    SaveEvidence(Vec<Evidence>),
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
                    set_label: &lang::lookup("flow-execution-running"),
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
                log::debug!("Output state: {outputs:?}");
                log::debug!("Evidence state: {evidence:?}");
                log::debug!("Executing: {action_config:?}");
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
                        return ExecutionDialogCommandOutput::Failed(step + 1, e, evidence);
                    }
                }
            }

            ExecutionDialogCommandOutput::Complete(evidence)
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            ExecutionDialogInput::Close => root.destroy(),
            ExecutionDialogInput::FailedToGenerateReport(reason) => {
                let dialog = self.create_message_dialog(
                    lang::lookup("report-failed"),
                    lang::lookup_with_args("report-failed-message", {
                        let mut map = HashMap::new();
                        map.insert("reason", reason.to_string().into());
                        map
                    }),
                );
                dialog.set_transient_for(Some(root));
                dialog.add_response("ok", &lang::lookup("ok"));
                dialog.set_default_response(Some("ok"));
                let sender_c = sender.clone();
                dialog.connect_response(None, move |dlg, _response| {
                    sender_c.input(ExecutionDialogInput::Close);
                    dlg.close();
                });
                dialog.set_visible(true);
            }
            ExecutionDialogInput::SaveEvidence(evidence) => {
                // Present save dialog
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("report-save-title"))
                    .initial_name(lang::lookup("report-default-name"))
                    .filters(&file_filters::filter_list(vec![
                        file_filters::pdfs(),
                        file_filters::all(),
                    ]))
                    .build();

                let sender_c = sender.clone();
                dialog.save(
                    Some(root),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            if let Err(e) = report_generation::save_report(
                                path.with_extension("pdf"),
                                evidence.clone(),
                            ) {
                                // Failed to generate report
                                sender_c.input(ExecutionDialogInput::FailedToGenerateReport(e));
                                return;
                            } else if let Err(e) = opener::open(path.with_extension("pdf")) {
                                log::warn!("Failed to open evidence: {e}");
                            }
                        }
                        sender_c.input(ExecutionDialogInput::Close);
                    },
                );
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
                sender.input(ExecutionDialogInput::SaveEvidence(evidence));
            }

            ExecutionDialogCommandOutput::Failed(step, reason, evidence) => {
                log::warn!("Execution failed. Evidence: {evidence:?}");
                let dialog = self.create_message_dialog(
                    lang::lookup("flow-execution-failed"),
                    lang::lookup_with_args("flow-execution-failed-message", {
                        let mut map = HashMap::new();
                        map.insert("step", step.into());
                        map.insert("reason", reason.to_string().into());
                        map
                    }),
                );
                dialog.set_transient_for(Some(root));
                if !evidence.is_empty() {
                    dialog
                        .add_response("save", &lang::lookup("flow-execution-save-evidence-anyway"));
                }
                dialog.add_response("ok", &lang::lookup("ok"));
                dialog.set_default_response(Some("ok"));
                let sender_c = sender.clone();
                dialog.connect_response(None, move |dlg, response| {
                    if response == "save" {
                        sender_c.input(ExecutionDialogInput::SaveEvidence(evidence.clone()));
                    }
                    sender_c.input(ExecutionDialogInput::Close);
                    dlg.close();
                });
                dialog.set_visible(true);
            }
        }
    }
}
