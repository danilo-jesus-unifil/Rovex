use super::super::context::AppContext;
use super::super::jobs::{SearchEvent, SearchScheduler};
use super::super::state::{self, LoadedRow, SharedRows, SharedSelection};
use crate::filesystem::ListingOptions;
use crate::search::{SearchLimits, SearchReport, SearchStatus};
use slint::{Model, SharedString};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::Ordering};

const SEARCH_LIMITS: SearchLimits = SearchLimits {
    max_results: 10_000,
    max_visited_directories: 100_000,
    max_visited_entries: 1_000_000,
};

#[derive(Clone)]
struct SearchContext {
    scheduler: Option<Arc<SearchScheduler>>,
    rows: SharedRows,
    selection: SharedSelection,
    generation: Arc<std::sync::atomic::AtomicU64>,
    options: Arc<Mutex<ListingOptions>>,
    sort_spec: Arc<Mutex<state::SortSpec>>,
}

fn status_for_report(report: &SearchReport) -> String {
    match report.status {
        SearchStatus::Completed => format!("Pesquisa concluída: {} resultado(s).", report.matches),
        SearchStatus::Cancelled => format!(
            "Pesquisa cancelada: {} resultado(s) parcial(is).",
            report.matches
        ),
        SearchStatus::Limited(limit) => format!(
            "Pesquisa limitada ({limit:?}): {} resultado(s), {} entrada(s) visitada(s).",
            report.matches, report.visited_entries
        ),
    }
}

fn clear_search_rows(
    ui: &super::super::MainWindow,
    rows: &SharedRows,
    selection: &SharedSelection,
) -> bool {
    let Ok(mut selection_state) = selection.lock() else {
        return false;
    };
    selection_state.clear();
    ui.set_selection_count(0);
    ui.set_focused_row_index(-1);
    let Ok(mut row_snapshot) = rows.lock() else {
        return false;
    };
    *row_snapshot = Arc::from(Vec::<LoadedRow>::new());
    state::set_rows(ui, Vec::new(), &selection_state)
}

fn publish_update(
    ui: &super::super::MainWindow,
    update: crate::search::SearchUpdate,
    rows: &SharedRows,
    selection: &SharedSelection,
    sort_spec: &Arc<Mutex<state::SortSpec>>,
) {
    let Ok(mut row_snapshot) = rows.lock() else {
        ui.set_status_text("Falha interna ao armazenar resultados da pesquisa".into());
        return;
    };
    let mut next = row_snapshot.as_ref().to_vec();
    let start_index = next.len();
    next.extend(
        update
            .batch
            .iter()
            .enumerate()
            .map(|(index, entry)| state::row_from_entry(entry, start_index + index)),
    );
    let current_sort = sort_spec.lock().map(|sort| *sort).unwrap_or_default();
    state::sort_rows(&mut next, current_sort);
    *row_snapshot = Arc::from(next.clone());
    drop(row_snapshot);
    let Ok(selection_state) = selection.lock() else {
        ui.set_status_text("Falha interna ao ler a seleção".into());
        return;
    };
    if !state::set_rows(ui, next, &selection_state) {
        ui.set_status_text("Falha interna ao atualizar resultados".into());
        return;
    }
    ui.set_empty_state_text(SharedString::default());
    ui.set_status_text(SharedString::from(format!(
        "Pesquisa: {} resultado(s), {} entrada(s) visitada(s).",
        ui.get_entries().row_count(),
        update.visited_entries
    )));
}

fn register_start(
    ui_weak: &slint::Weak<super::super::MainWindow>,
    context: &SearchContext,
    ui: &super::super::MainWindow,
) {
    let scheduler = context.scheduler.clone();
    let rows = Arc::clone(&context.rows);
    let selection = Arc::clone(&context.selection);
    let generation = Arc::clone(&context.generation);
    let options = Arc::clone(&context.options);
    let sort_spec = Arc::clone(&context.sort_spec);
    let query = ui.get_filter_text().to_string();
    if query.trim().is_empty() {
        ui.set_status_text("Informe um nome para pesquisar nas subpastas.".into());
        return;
    }
    let root = PathBuf::from(ui.get_current_path().to_string());
    let current_generation = generation.fetch_add(1, Ordering::AcqRel) + 1;
    let Some(scheduler) = scheduler else {
        ui.set_status_text("Pesquisa indisponível.".into());
        return;
    };
    if !clear_search_rows(ui, &rows, &selection) {
        ui.set_status_text("Falha interna ao limpar resultados anteriores.".into());
        return;
    }
    ui.set_search_active(true);
    ui.set_status_text("Pesquisando nas subpastas…".into());
    let listing_options = options.lock().map(|value| *value).unwrap_or_default();
    let rows_for_callback = Arc::clone(&rows);
    let selection_for_callback = Arc::clone(&selection);
    let generation_for_callback = Arc::clone(&generation);
    let sort_for_callback = Arc::clone(&sort_spec);
    let ui_for_callback = ui_weak.clone();
    scheduler
        .start(
            current_generation,
            root,
            query,
            listing_options,
            SEARCH_LIMITS,
            move |event| {
                let generation_for_event = Arc::clone(&generation_for_callback);
                let rows = Arc::clone(&rows_for_callback);
                let selection = Arc::clone(&selection_for_callback);
                let sort_spec = Arc::clone(&sort_for_callback);
                let _ = ui_for_callback.upgrade_in_event_loop(move |ui| {
                    if generation_for_event.load(Ordering::Acquire) != current_generation {
                        return;
                    }
                    let event_generation = match &event {
                        SearchEvent::Update { generation, .. }
                        | SearchEvent::Finished { generation, .. }
                        | SearchEvent::Failed { generation, .. } => *generation,
                    };
                    if event_generation != current_generation {
                        return;
                    }
                    match event {
                        SearchEvent::Update { update, .. } => {
                            if let Some(status) = update.status {
                                ui.set_status_text(SharedString::from(format!(
                                    "Pesquisa: {status:?}."
                                )));
                            }
                            publish_update(&ui, update, &rows, &selection, &sort_spec);
                        }
                        SearchEvent::Finished { report, .. } => {
                            ui.set_search_active(false);
                            ui.set_status_text(SharedString::from(status_for_report(&report)));
                            if report.matches == 0 {
                                ui.set_empty_state_text(
                                    "Nenhum item encontrado nas subpastas.".into(),
                                );
                            }
                        }
                        SearchEvent::Failed { error, .. } => {
                            ui.set_search_active(false);
                            ui.set_status_text(SharedString::from(format!(
                                "Pesquisa falhou: {error}"
                            )));
                        }
                    }
                });
            },
        )
        .unwrap_or_else(|_| {
            ui.set_search_active(false);
            ui.set_status_text("Falha ao iniciar a pesquisa.".into());
        });
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let context = SearchContext {
        scheduler: ctx.search_scheduler.clone(),
        rows: Arc::clone(&ctx.directory_rows),
        selection: Arc::clone(&ctx.selection),
        generation: Arc::clone(&ctx.filter_generation),
        options: Arc::clone(&ctx.listing_options),
        sort_spec: Arc::clone(&ctx.sort_spec),
    };
    {
        let ui_weak = ui_weak.clone();
        let context = context.clone();
        registration_ui.on_recursive_search_requested(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            register_start(&ui_weak, &context, &ui);
        });
    }
    registration_ui.on_recursive_search_cancelled(move || {
        if let Some(scheduler) = context.scheduler.as_ref() {
            scheduler.cancel();
        }
        context.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_search_active(false);
            ui.set_status_text("Pesquisa cancelada.".into());
        }
    });
}
