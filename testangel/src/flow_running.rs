use std::{collections::HashMap, fmt, fs, io::Cursor, sync::Arc, thread};

use base64::Engine;
use egui_file::FileDialog;
use genpdf::{
    elements,
    style::{Style, StyledString},
};
use testangel_ipc::prelude::*;

use crate::{
    action_loader::ActionMap,
    automation_flow::types::AutomationFlow,
    ipc::{EngineList, IpcError},
    UiComponent,
};

#[derive(Default)]
pub struct FlowRunningState {
    action_map: Arc<ActionMap>,
    engine_map: Arc<EngineList>,
    pub flow: Option<AutomationFlow>,
    running: bool,
    num_dots_ellipsis: f32,
    save_dialog: Option<FileDialog>,
    worker_thread: Option<thread::JoinHandle<Result<FlowExecutionResult, FlowError>>>,
}

impl FlowRunningState {
    pub fn new(action_map: Arc<ActionMap>, engine_map: Arc<EngineList>) -> Self {
        Self {
            action_map,
            engine_map,
            ..Default::default()
        }
    }

    /// Start execution of this automation flow.
    pub fn start_flow(&mut self) {
        self.running = true;
        let flow = self.flow.as_ref().unwrap().clone();
        let action_map = self.action_map.clone();
        let engine_map = self.engine_map.clone();

        self.worker_thread = Some(thread::spawn(move || {
            let flow = flow;
            let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
            let mut evidence = Vec::new();

            for engine in engine_map.inner() {
                if let Ok(_) = engine.reset_state() {
                    evidence.push(Evidence {
                        label: String::from("WARNING: State Warning"),
                        content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
                    });
                }
            }

            for action_config in flow.actions {
                let (output, ev) = action_config.execute(
                    action_map.clone(),
                    engine_map.clone(),
                    outputs.clone(),
                )?;
                outputs.push(output);
                evidence = vec![evidence, ev].concat();
            }
            Ok(FlowExecutionResult { evidence })
        }));
    }
}

impl UiComponent for FlowRunningState {
    fn menu_bar(&mut self, _ui: &mut egui::Ui) -> Option<crate::State> {
        None
    }

    fn always_ui(&mut self, ctx: &egui::Context) -> Option<crate::State> {
        if let Some(dialog) = &mut self.save_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    // generate pdf and save
                    // TODO improve error handling
                    fs::create_dir_all("./.tafonts").unwrap();
                    fs::write(
                        "./.tafonts/LiberationSans-Bold.ttf",
                        include_bytes!("./fonts/LiberationSans-Bold.ttf"),
                    )
                    .unwrap();
                    fs::write(
                        "./.tafonts/LiberationSans-BoldItalic.ttf",
                        include_bytes!("./fonts/LiberationSans-BoldItalic.ttf"),
                    )
                    .unwrap();
                    fs::write(
                        "./.tafonts/LiberationSans-Italic.ttf",
                        include_bytes!("./fonts/LiberationSans-Italic.ttf"),
                    )
                    .unwrap();
                    fs::write(
                        "./.tafonts/LiberationSans-Regular.ttf",
                        include_bytes!("./fonts/LiberationSans-Regular.ttf"),
                    )
                    .unwrap();

                    let font_family =
                        genpdf::fonts::from_files("./.tafonts", "LiberationSans", None).unwrap();
                    let mut doc = genpdf::Document::new(font_family);
                    doc.set_title("TestAngel Evidence");
                    let mut decorator = genpdf::SimplePageDecorator::new();
                    decorator.set_margins(10);
                    decorator.set_header(|page_no| {
                        elements::PaddedElement::new(
                            elements::LinearLayout::vertical()
                                .element(elements::Text::new(StyledString::new(
                                    "Flow Evidence",
                                    Style::new().bold().with_font_size(18),
                                )))
                                .element(elements::Text::new(StyledString::new(
                                    format!(
                                        "Page {page_no} - Generated by TestAngel at {}",
                                        chrono::Local::now().format("%Y-%m-%d %H:%M")
                                    ),
                                    Style::new().with_font_size(10),
                                ))),
                            (0, 0, 4, 0),
                        )
                    });
                    doc.set_page_decorator(decorator);

                    let res = self.worker_thread.take().unwrap().join().unwrap();
                    if let Ok(fer) = res {
                        for ev in &fer.evidence {
                            doc.push(elements::Paragraph::new(ev.label.clone()));
                            match &ev.content {
                                EvidenceContent::Textual(text) => {
                                    doc.push(elements::Paragraph::new(text))
                                }
                                EvidenceContent::ImageAsPngBase64(base64) => {
                                    let data = base64::engine::general_purpose::STANDARD
                                        .decode(base64)
                                        .unwrap();
                                    doc.push(
                                        elements::Image::from_reader(Cursor::new(data)).unwrap(),
                                    );
                                }
                            }
                        }
                    } else if let Err(fe) = res {
                        doc.push(elements::Paragraph::new("An execution error occurred:"));
                        doc.push(elements::Paragraph::new(format!("{fe}")));
                    }

                    doc.render_to_file(path.with_extension("pdf")).unwrap();
                    fs::remove_dir_all("./.tafonts").unwrap();

                    // Set the worker thread to None, the flow to None, and return a new state of the editor.
                    self.worker_thread = None;
                    self.flow = None;
                    return Some(crate::State::AutomationFlowEditor);
                }
            }
        }

        None
    }

    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<crate::State> {
        if self.running {
            self.num_dots_ellipsis =
                ctx.animate_value_with_time(egui::Id::new("flowrunning-ellipses"), 3.99, 3.0);
            if self.num_dots_ellipsis == 3.99 {
                self.num_dots_ellipsis =
                    ctx.animate_value_with_time(egui::Id::new("flowrunning-ellipses"), 0.0, 0.0);
            }

            let mut ellipses = String::new();
            let num_dots = self.num_dots_ellipsis.floor() as i32;
            for _ in 0..num_dots {
                ellipses.push('.');
            }
            ui.heading(format!("Automation flow running{ellipses}"));

            if let Some(handle) = &self.worker_thread {
                if handle.is_finished() {
                    self.running = false;
                }
            }
        } else {
            ui.heading("Saving Automation Flow execution report.");
            if let None = self.save_dialog {
                let mut dialog = FileDialog::save_file(None);
                dialog.open();
                self.save_dialog = Some(dialog);
            }
        }

        None
    }
}

struct FlowExecutionResult {
    evidence: Vec<Evidence>,
}

pub enum FlowError {
    FromInstruction {
        error_kind: ErrorKind,
        reason: String,
    },
    IPCFailure(IpcError),
}

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IPCFailure(e) => write!(f, "An IPC call failed ({e:?})."),
            Self::FromInstruction { error_kind, reason } => write!(
                f,
                "An instruction returned an error: {error_kind:?}: {reason}"
            ),
        }
    }
}
