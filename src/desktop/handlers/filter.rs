use super::super::context::AppContext;
use std::sync::{Arc, atomic::Ordering};

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let filter_generation = Arc::clone(&ctx.filter_generation);
    let filter_scheduler = ctx.filter_scheduler.clone();
    let selection = Arc::clone(&ctx.selection);
    {
        let ui_weak = ui_weak.clone();
        let filter_generation = Arc::clone(&filter_generation);
        let filter_scheduler = filter_scheduler.clone();
        let selection = Arc::clone(&selection);
        ui.on_filter_changed(move |text| {
            if let Ok(mut state) = selection.lock() {
                state.clear();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_selection_count(0);
                    ui.set_focused_row_index(-1);
                }
            }
            let generation = filter_generation.fetch_add(1, Ordering::AcqRel) + 1;
            let query = text.to_string();
            let Some(scheduler) = filter_scheduler.as_ref() else {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Filtro indisponível".into());
                });
                return;
            };
            if scheduler.schedule(generation, query).is_err() {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Falha ao agendar o filtro".into());
                });
            }
        });
    }
}
