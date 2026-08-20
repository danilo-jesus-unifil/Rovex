use super::super::MainWindow;
use super::super::context::AppContext;
use super::super::state::{LoadedRow, SharedRows, SharedSelection};
use crate::activation::activate_file;
use slint::{SharedString, Weak};
use std::path::PathBuf;
use std::sync::Arc;

fn selected_row(rows: &SharedRows, selection: &SharedSelection) -> Option<LoadedRow> {
    let selected = selection.lock().ok()?.selected.clone();
    if selected.len() != 1 {
        return None;
    }
    let rows = rows.lock().ok()?;
    rows.iter().find(|row| selected.contains(&row.key)).cloned()
}

pub(in crate::desktop) fn is_activatable_row(row: &LoadedRow) -> bool {
    !row.is_directory && matches!(row.kind.as_str(), "Arquivo" | "Arquivo oculto")
}

pub(in crate::desktop) fn spawn_file_activation(ui_weak: &Weak<MainWindow>, path: PathBuf) {
    let callback_ui = ui_weak.clone();
    let worker = std::thread::Builder::new()
        .name("rovex-file-activation".to_owned())
        .spawn(move || {
            let message = match activate_file(&path) {
                Ok(()) => format!("Arquivo aberto com o aplicativo padrão: {}", path.display()),
                Err(error) => error.to_string(),
            };
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = callback_ui.upgrade() {
                    ui.set_status_text(SharedString::from(message));
                }
            });
        });
    if worker.is_err() {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Não foi possível iniciar o worker de ativação".into());
        }
    } else if let Some(ui) = ui_weak.upgrade() {
        ui.set_status_text("Abrindo com o aplicativo padrão…".into());
    }
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);

    registration_ui.on_context_menu_open_requested(move || {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        ui.set_context_menu_visible(false);
        let Some(row) = selected_row(&rows, &selection) else {
            ui.set_status_text("Selecione exatamente um arquivo para abrir".into());
            return;
        };
        if !is_activatable_row(&row) {
            ui.set_status_text("Abrir exige um arquivo regular".into());
            return;
        }
        if !crate::activation::is_supported() {
            ui.set_status_text("Abrir arquivo não está disponível nesta plataforma".into());
            return;
        }
        spawn_file_activation(&ui_weak, row.path);
    });
}
