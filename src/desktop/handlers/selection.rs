use super::super::context::AppContext;
use super::super::state::{selection_status, update_selection_visuals};
use super::preview;
use crate::converters::ConversionKind;
use slint::{Model, SharedString};
use std::path::Path;
use std::sync::Arc;

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let entries = ctx.entries.clone();
    let selection = Arc::clone(&ctx.selection);
    let rows = Arc::clone(&ctx.directory_rows);
    let preview_scheduler = ctx.preview_scheduler.clone();
    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        let rows = Arc::clone(&rows);
        let preview_scheduler = preview_scheduler.clone();
        ui.on_select_row(move |index, control, shift| {
            if index < 0 {
                return;
            }
            let Some(row) = entries.row_data(index as usize) else {
                return;
            };
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|visible_row| visible_row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.click(row.key.as_str(), &keys, control, shift);
            if let Some(ui) = ui_weak.upgrade() {
                if !update_selection_visuals(&ui, &state) {
                    ui.set_status_text("Falha interna ao atualizar a seleção".into());
                } else {
                    ui.set_selection_count(state.count() as i32);
                    ui.set_status_text(SharedString::from(selection_status(&state)));
                }
            }
            drop(state);
            preview::refresh_selection(&ui_weak, preview_scheduler.as_ref(), &rows, &selection);
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        let rows = Arc::clone(&rows);
        let preview_scheduler = preview_scheduler.clone();
        ui.on_select_all(move || {
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|row| row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.select_all(keys);
            if let Some(ui) = ui_weak.upgrade() {
                if !update_selection_visuals(&ui, &state) {
                    ui.set_status_text("Falha interna ao atualizar a seleção".into());
                } else {
                    ui.set_selection_count(state.count() as i32);
                    ui.set_status_text(SharedString::from(selection_status(&state)));
                }
            }
            drop(state);
            preview::refresh_selection(&ui_weak, preview_scheduler.as_ref(), &rows, &selection);
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        let rows = Arc::clone(&rows);
        let preview_scheduler = preview_scheduler.clone();
        ui.on_context_menu_requested(move |index| {
            if index < 0 {
                return;
            }
            let Some(row) = entries.row_data(index as usize) else {
                return;
            };
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|visible_row| visible_row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.click(row.key.as_str(), &keys, false, false);
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            if !update_selection_visuals(&ui, &state) {
                ui.set_status_text("Falha interna ao atualizar a seleção".into());
                return;
            }
            ui.set_selection_count(state.count() as i32);
            ui.set_status_text(SharedString::from(selection_status(&state)));
            let is_regular_file = row.kind == "Arquivo";
            ui.set_context_menu_target_name(row.name.clone());
            ui.set_context_menu_can_jxl(
                is_regular_file && ConversionKind::JpegXl.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_opus(
                is_regular_file && ConversionKind::Opus.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_png(
                is_regular_file && ConversionKind::Png.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_flac(
                is_regular_file && ConversionKind::Flac.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_terminal(crate::terminal::is_supported());
            ui.set_context_menu_visible(true);
            drop(state);
            preview::refresh_selection(&ui_weak, preview_scheduler.as_ref(), &rows, &selection);
        });
    }
}
