use super::super::context::AppContext;
use std::sync::{Arc, atomic::Ordering};

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let filter_generation = Arc::clone(&ctx.filter_generation);
    let filter_scheduler = ctx.filter_scheduler.clone();
    let selection = Arc::clone(&ctx.selection);
    let sort_spec = Arc::clone(&ctx.sort_spec);
    let save_settings = ctx.settings_saver();

    registration_ui.on_sort_requested(move |column| {
        let Some(ui) = ui_weak.upgrade() else {
            return;
        };
        let next_sort = {
            let Ok(mut current_sort) = sort_spec.lock() else {
                ui.set_status_text("Falha interna ao trocar a ordenação".into());
                return;
            };
            let next_sort = current_sort.toggle_column(column);
            *current_sort = next_sort;
            next_sort
        };
        ui.set_sort_column(next_sort.field.column());
        ui.set_sort_ascending(next_sort.direction.is_ascending());
        save_settings();
        if let Ok(mut state) = selection.lock() {
            state.clear();
            ui.set_selection_count(0);
            ui.set_focused_row_index(-1);
        }

        let generation = filter_generation.fetch_add(1, Ordering::AcqRel) + 1;
        let query = ui.get_filter_text().to_string();
        let Some(scheduler) = filter_scheduler.as_ref() else {
            ui.set_status_text("Ordenação indisponível".into());
            return;
        };
        if scheduler.schedule(generation, query, next_sort).is_err() {
            let _ = ui_weak.upgrade_in_event_loop(|ui| {
                ui.set_status_text("Falha ao agendar a ordenação".into());
            });
        }
    });
}
