use super::super::MainWindow;
use super::super::state::{self, LoadedRow, SharedRows, SharedSelection, SortSpec};
use slint::SharedString;
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;

struct LoadRequest {
    generation: u64,
    path: PathBuf,
}

pub(in crate::desktop) struct LoadScheduler {
    pending: Arc<(Mutex<Option<LoadRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
    load_generation: Arc<AtomicU64>,
    filter_generation: Arc<AtomicU64>,
}

impl LoadScheduler {
    pub(in crate::desktop) fn new(
        ui_weak: slint::Weak<MainWindow>,
        directory_rows: SharedRows,
        selection: SharedSelection,
        filter_generation: Arc<AtomicU64>,
        sort_spec: Arc<Mutex<SortSpec>>,
    ) -> Result<Self, ()> {
        let pending = Arc::new((Mutex::new(None::<LoadRequest>), std::sync::Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let load_generation = Arc::new(AtomicU64::new(0));
        let worker_pending = Arc::clone(&pending);
        let worker_stop = Arc::clone(&stop);
        let worker_load_generation = Arc::clone(&load_generation);
        let worker_filter_generation = Arc::clone(&filter_generation);
        let worker_directory_rows = Arc::clone(&directory_rows);
        let worker_selection = Arc::clone(&selection);
        let worker_sort_spec = Arc::clone(&sort_spec);
        thread::Builder::new()
            .name("rovex-filesystem-loader".to_owned())
            .spawn(move || {
                loop {
                    let request = {
                        let (lock, condition) = &*worker_pending;
                        let Ok(pending) = lock.lock() else {
                            break;
                        };
                        let mut pending = match condition.wait_while(pending, |request| {
                            request.is_none() && !worker_stop.load(Ordering::Acquire)
                        }) {
                            Ok(pending) => pending,
                            Err(_) => break,
                        };
                        if pending.is_none() && worker_stop.load(Ordering::Acquire) {
                            break;
                        }
                        pending.take()
                    };

                    let Some(request) = request else {
                        continue;
                    };
                    let loaded = state::load_directory(request.path);
                    let ui_load_generation = Arc::clone(&worker_load_generation);
                    let ui_filter_generation = Arc::clone(&worker_filter_generation);
                    let ui_directory_rows = Arc::clone(&worker_directory_rows);
                    let ui_selection = Arc::clone(&worker_selection);
                    let ui_sort_spec = Arc::clone(&worker_sort_spec);
                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        if ui_load_generation.load(Ordering::Acquire) != request.generation {
                            return;
                        }
                        let current_sort =
                            ui_sort_spec.lock().map(|sort| *sort).unwrap_or_default();
                        let mut loaded_rows = loaded.rows;
                        state::sort_rows(&mut loaded_rows, current_sort);
                        ui.set_sort_column(current_sort.field.column());
                        ui.set_sort_ascending(current_sort.direction.is_ascending());
                        ui.set_current_path(SharedString::from(
                            loaded.path.to_string_lossy().to_string(),
                        ));
                        ui_filter_generation.fetch_add(1, Ordering::AcqRel);
                        ui.set_filter_text(SharedString::default());
                        let empty_state = if loaded.is_error {
                            ""
                        } else {
                            state::empty_state_text(loaded_rows.len(), loaded_rows.len(), "")
                        };
                        ui.set_empty_state_text(SharedString::from(empty_state));
                        ui.set_focused_row_index(-1);
                        let Ok(mut selection_state) = ui_selection.lock() else {
                            ui.set_status_text("Falha interna ao limpar a seleção".into());
                            return;
                        };
                        selection_state.clear();
                        ui.set_selection_count(0);
                        let snapshot: Arc<[LoadedRow]> = Arc::from(loaded_rows);
                        let Ok(mut rows) = ui_directory_rows.lock() else {
                            ui.set_status_text("Falha interna ao armazenar a listagem".into());
                            return;
                        };
                        *rows = Arc::clone(&snapshot);
                        ui.set_status_text(SharedString::from(loaded.status));
                        if !state::set_rows(&ui, snapshot.as_ref().to_vec(), &selection_state) {
                            ui.set_status_text("Falha interna ao atualizar a lista".into());
                        }
                    });
                }
            })
            .map_err(|_| ())?;

        Ok(Self {
            pending,
            stop,
            load_generation,
            filter_generation,
        })
    }

    pub(in crate::desktop) fn schedule(&self, path: PathBuf) -> Result<(), ()> {
        let generation = self.load_generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.filter_generation.fetch_add(1, Ordering::AcqRel);
        let (lock, condition) = &*self.pending;
        let Ok(mut pending) = lock.lock() else {
            return Err(());
        };
        *pending = Some(LoadRequest { generation, path });
        condition.notify_one();
        Ok(())
    }
}

impl Drop for LoadScheduler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let (_, condition) = &*self.pending;
        condition.notify_one();
    }
}

pub(in crate::desktop) fn start_load(
    ui_weak: &slint::Weak<MainWindow>,
    path: PathBuf,
    scheduler: Option<&Arc<LoadScheduler>>,
) {
    let Some(scheduler) = scheduler else {
        let _ = ui_weak.upgrade_in_event_loop(|ui| {
            ui.set_status_text("Carregador indisponível".into());
        });
        return;
    };
    if scheduler.schedule(path).is_err() {
        let _ = ui_weak.upgrade_in_event_loop(|ui| {
            ui.set_status_text("Falha ao agendar o carregamento".into());
        });
    }
}
