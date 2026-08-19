use super::super::context::AppContext;
use super::super::state::{LoadedRow, SharedRows, SharedSelection};
use crate::open_with::open_with_file;
use slint::SharedString;
use std::sync::Arc;

fn selected_row(rows: &SharedRows, selection: &SharedSelection) -> Option<LoadedRow> {
    let selected = selection.lock().ok()?.selected.clone();
    if selected.len() != 1 {
        return None;
    }
    let rows = rows.lock().ok()?;
    rows.iter().find(|row| selected.contains(&row.key)).cloned()
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);

    registration_ui.on_context_menu_open_with_requested(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        ui.set_context_menu_visible(false);
        let Some(row) = selected_row(&rows, &selection) else {
            ui.set_status_text("Selecione exatamente um arquivo para usar Open With".into());
            return;
        };
        if row.is_directory || row.kind != "Arquivo" {
            ui.set_status_text("Open With exige um arquivo regular".into());
            return;
        }
        let path = row.path.clone();
        let callback_ui = ui_weak.clone();
        let worker = std::thread::Builder::new()
            .name("rovex-open-with".to_owned())
            .spawn(move || {
                let message = match open_with_file(&path) {
                    Ok(()) => format!("Diálogo Open With concluído para {}", path.display()),
                    Err(error) => error.to_string(),
                };
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = callback_ui.upgrade() {
                        ui.set_status_text(SharedString::from(message));
                    }
                });
            });
        if worker.is_err() {
            ui.set_status_text("Não foi possível iniciar o worker Open With".into());
        } else {
            ui.set_status_text("Abrindo diálogo Open With…".into());
        }
    });
}
