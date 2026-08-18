use super::super::context::AppContext;
use super::preview;
use std::sync::{Arc, atomic::Ordering};

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let filter_generation = Arc::clone(&ctx.filter_generation);
    let filter_scheduler = ctx.filter_scheduler.clone();
    let search_scheduler = ctx.search_scheduler.clone();
    let selection = Arc::clone(&ctx.selection);
    let rows = Arc::clone(&ctx.directory_rows);
    let preview_scheduler = ctx.preview_scheduler.clone();
    let sort_spec = Arc::clone(&ctx.sort_spec);
    {
        let ui_weak = ui_weak.clone();
        let filter_generation = Arc::clone(&filter_generation);
        let selection = Arc::clone(&selection);
        let rows = Arc::clone(&rows);
        let preview_scheduler = preview_scheduler.clone();
        let search_scheduler = search_scheduler.clone();
        let sort_spec = Arc::clone(&sort_spec);
        ui.on_filter_changed(move |text| {
            if let Some(search_scheduler) = search_scheduler.as_ref()
                && ui_weak.upgrade().is_some_and(|ui| ui.get_search_active())
            {
                search_scheduler.cancel();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_search_active(false);
                    ui.set_status_text("Pesquisa cancelada; filtro local aplicado.".into());
                }
            }
            if let Ok(mut state) = selection.lock() {
                state.clear();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_selection_count(0);
                    ui.set_focused_row_index(-1);
                }
            }
            preview::refresh_selection(&ui_weak, preview_scheduler.as_ref(), &rows, &selection);
            let generation = filter_generation.fetch_add(1, Ordering::AcqRel) + 1;
            let query = text.to_string();
            let current_sort = sort_spec.lock().map(|sort| *sort).unwrap_or_default();
            let Some(scheduler) = filter_scheduler.as_ref() else {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Filtro indisponível".into());
                });
                return;
            };
            if scheduler.schedule(generation, query, current_sort).is_err() {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Falha ao agendar o filtro".into());
                });
            }
        });
    }
}
