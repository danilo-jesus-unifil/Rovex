use super::super::context::AppContext;
use super::super::jobs::OperationKind;
use super::dialogs;
use std::sync::Arc;

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let pending_operation = ctx.pending_operation.clone();
    let directory_rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);
    let tabs = ctx.tabs.clone();
    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_copy_requested(move || {
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_move_requested(move || {
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_rename_requested(move || {
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_delete_requested(move || {
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Delete,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_copy_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_move_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_rename_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_delete_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            dialogs::show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Delete,
            );
        });
    }
}
