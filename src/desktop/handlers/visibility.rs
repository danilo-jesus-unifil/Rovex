use super::super::context::AppContext;
use super::super::jobs::start_load;
use std::sync::Arc;

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let listing_options = Arc::clone(&ctx.listing_options);
    let load_scheduler = ctx.load_scheduler.clone();
    let save_settings = ctx.settings_saver();

    registration_ui.on_toggle_hidden_files_requested(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        let show_hidden = {
            let Ok(mut options) = listing_options.lock() else {
                ui.set_status_text("Falha interna ao alterar a visibilidade".into());
                return;
            };
            options.show_hidden = !options.show_hidden;
            options.show_system = options.show_hidden;
            options.show_hidden
        };
        ui.set_show_hidden_files(show_hidden);
        save_settings();
        let path = std::path::PathBuf::from(ui.get_current_path().to_string());
        start_load(&ui_weak, path, load_scheduler.as_ref());
    });
}
