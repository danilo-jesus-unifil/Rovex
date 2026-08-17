use super::super::MainWindow;
use super::super::state::{self, SharedRows, SharedSelection};
use slint::SharedString;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;

struct FilterRequest {
    generation: u64,
    query: String,
}

pub(in crate::desktop) struct FilterScheduler {
    pending: Arc<(Mutex<Option<FilterRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
}

impl FilterScheduler {
    pub(in crate::desktop) fn new(
        ui_weak: slint::Weak<MainWindow>,
        directory_rows: SharedRows,
        selection: SharedSelection,
        filter_generation: Arc<AtomicU64>,
    ) -> Result<Self, ()> {
        let pending = Arc::new((Mutex::new(None::<FilterRequest>), std::sync::Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_pending = Arc::clone(&pending);
        let worker_stop = Arc::clone(&stop);
        thread::Builder::new()
            .name("rovex-filter-worker".to_owned())
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
                    let rows = match directory_rows.lock() {
                        Ok(rows) => Some(Arc::clone(&rows)),
                        Err(_) => None,
                    };
                    let result = match rows {
                        Some(rows) => {
                            let filtered = state::filter_rows(rows.as_ref(), &request.query);
                            let status =
                                state::filter_status(rows.len(), filtered.len(), &request.query);
                            let empty_state =
                                state::empty_state_text(rows.len(), filtered.len(), &request.query);
                            (Some(filtered), status, empty_state)
                        }
                        None => (None, "Falha interna ao ler a listagem".to_owned(), ""),
                    };
                    let ui_filter_generation = Arc::clone(&filter_generation);
                    let ui_selection = Arc::clone(&selection);
                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        if ui_filter_generation.load(Ordering::Acquire) != request.generation {
                            return;
                        }
                        let (filtered, status, empty_state) = result;
                        let Ok(selection) = ui_selection.lock() else {
                            ui.set_status_text("Falha interna ao ler a seleção".into());
                            return;
                        };
                        ui.set_empty_state_text(SharedString::from(empty_state));
                        ui.set_status_text(SharedString::from(status));
                        if let Some(filtered) = filtered
                            && !state::set_rows(&ui, filtered, &selection)
                        {
                            ui.set_status_text("Falha interna ao atualizar a lista".into());
                        }
                    });
                }
            })
            .map_err(|_| ())?;

        Ok(Self { pending, stop })
    }

    pub(in crate::desktop) fn schedule(&self, generation: u64, query: String) -> Result<(), ()> {
        let (lock, condition) = &*self.pending;
        let Ok(mut pending) = lock.lock() else {
            return Err(());
        };
        *pending = Some(FilterRequest { generation, query });
        condition.notify_one();
        Ok(())
    }
}

impl Drop for FilterScheduler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let (_, condition) = &*self.pending;
        condition.notify_one();
    }
}
