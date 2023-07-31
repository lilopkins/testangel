fn main() {
    pretty_env_logger::init();

    let native_options = eframe::NativeOptions::default();
    // TODO: Create icon for app
    // native_options.icon_data = Some(IconData::try_from_png_bytes(include_bytes!("icon.png")).unwrap());
    if let Err(err) = eframe::run_native("TestAngel", native_options, Box::new(|cc| Box::new(App::new(cc)))) {
        log::error!("Error initialising window: {err}");
    }
}

#[derive(Default)]
struct App;

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for App {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       egui::CentralPanel::default().show(ctx, |ui| {
           ui.heading("Hello World!");
       });
   }
}
