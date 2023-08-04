use egui_modal::{Icon, Modal};

pub fn about_modal(ctx: &egui::Context) -> Modal {
    let about_modal = Modal::new(ctx, "about_modal");
    about_modal.show(|ui| {
        about_modal.title(ui, "About TestAngel");
        about_modal.frame(ui, |ui| {
            about_modal.body_and_icon(ui, "TestAngel automates testing across a number of tools by providing a standardised interface to communicate actions to perform.", Icon::Info);
        });
        about_modal.buttons(ui, |ui| {
            let _ = about_modal.button(ui, "Close");
        });
    });
    about_modal
}

pub fn error_modal<S: AsRef<str>>(ctx: &egui::Context, id: S, error: S) -> Modal {
    let error_modal = Modal::new(ctx, id.as_ref());
    error_modal.show(|ui| {
        error_modal.title(ui, "Error");
        error_modal.frame(ui, |ui| {
            error_modal.body_and_icon(ui, error.as_ref(), Icon::Error);
        });
        error_modal.buttons(ui, |ui| {
            let _ = error_modal.button(ui, "OK");
        });
    });
    error_modal
}
