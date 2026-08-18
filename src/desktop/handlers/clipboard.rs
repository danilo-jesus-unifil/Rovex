use super::super::context::AppContext;
use super::super::jobs::{OperationKind, OperationRequest};
use super::super::state::{SharedRows, SharedSelection, TabManager, selected_paths};
use crate::clipboard::{ClipboardAction, ClipboardError, ClipboardStore};
use slint::SharedString;
use std::rc::Rc;
use std::sync::Arc;

fn set_clipboard_status(ui: &super::super::MainWindow, action: ClipboardAction, count: usize) {
    let verb = match action {
        ClipboardAction::Copy => "copiados",
        ClipboardAction::Cut => "recortados",
    };
    ui.set_status_text(SharedString::from(format!(
        "{count} item(ns) {verb} para o clipboard"
    )));
}

fn report_clipboard_error(ui: &super::super::MainWindow, error: ClipboardError) {
    ui.set_status_text(SharedString::from(error.to_string()));
}

fn set_selection_clipboard(
    ui: &super::super::MainWindow,
    rows: &SharedRows,
    selection: &SharedSelection,
    store: Option<&ClipboardStore>,
    action: ClipboardAction,
) {
    let paths = selected_paths(rows, selection);
    let count = paths.len();
    let Some(store) = store else {
        report_clipboard_error(ui, ClipboardError::Backend("backend indisponível".into()));
        return;
    };
    match store.set_paths(paths, action) {
        Ok(()) => set_clipboard_status(ui, action, count),
        Err(error) => report_clipboard_error(ui, error),
    }
}

fn start_paste(
    ui: &super::super::MainWindow,
    tabs: &Rc<std::cell::RefCell<TabManager>>,
    store: Option<&ClipboardStore>,
    scheduler: Option<&Arc<super::super::jobs::OperationScheduler>>,
) {
    let Some(store) = store else {
        report_clipboard_error(ui, ClipboardError::Backend("backend indisponível".into()));
        return;
    };
    let payload = match store.get_payload() {
        Ok(payload) => payload,
        Err(error) => {
            report_clipboard_error(ui, error);
            return;
        }
    };
    let Some(scheduler) = scheduler else {
        ui.set_status_text("O worker de operações está indisponível.".into());
        return;
    };
    let refresh_path = tabs.borrow().active().current.clone();
    let kind = match payload.action {
        ClipboardAction::Copy => OperationKind::Copy,
        ClipboardAction::Cut => OperationKind::Move,
    };
    let count = payload.paths.len();
    let request = OperationRequest {
        kind,
        sources: payload.paths,
        destination_directory: Some(refresh_path.clone()),
        rename_name: None,
        refresh_path,
    };
    if scheduler.start(request).is_err() {
        ui.set_status_text("Já existe uma operação em andamento; aguarde o resultado.".into());
        return;
    }
    ui.set_operation_dialog_title("Colar itens".into());
    ui.set_operation_dialog_message(SharedString::from(format!(
        "Colando {count} item(ns) nesta pasta."
    )));
    ui.set_operation_dialog_input(SharedString::default());
    ui.set_operation_needs_input(false);
    ui.set_operation_close_only(false);
    ui.set_operation_busy(true);
    ui.set_operation_progress(0);
    ui.set_operation_progress_text("Preparando…".into());
    ui.set_operation_dialog_visible(true);
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);
    let tabs = ctx.tabs.clone();
    let scheduler = ctx.operation_scheduler.clone();
    let store = ctx.clipboard.clone();

    {
        let rows = Arc::clone(&rows);
        let store = store.clone();
        let selection = Arc::clone(&selection);
        let ui_weak = ui_weak.clone();
        registration_ui.on_clipboard_copy_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                set_selection_clipboard(
                    &ui,
                    &rows,
                    &selection,
                    store.as_deref(),
                    ClipboardAction::Copy,
                );
            }
        });
    }
    {
        let rows = Arc::clone(&rows);
        let selection = Arc::clone(&selection);
        let store = store.clone();
        let ui_weak = ui_weak.clone();
        registration_ui.on_clipboard_cut_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                set_selection_clipboard(
                    &ui,
                    &rows,
                    &selection,
                    store.as_deref(),
                    ClipboardAction::Cut,
                );
            }
        });
    }
    registration_ui.on_clipboard_paste_requested(move || {
        if let Some(ui) = ui_weak.upgrade() {
            start_paste(&ui, &tabs, store.as_deref(), scheduler.as_ref());
        }
    });
}
