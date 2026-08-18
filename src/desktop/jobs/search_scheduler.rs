use crate::filesystem::ListingOptions;
use crate::search::{SearchLimits, SearchReport, SearchUpdate, search_by_name};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

#[derive(Debug)]
pub(in crate::desktop) enum SearchEvent {
    Update {
        generation: u64,
        update: SearchUpdate,
    },
    Finished {
        generation: u64,
        report: SearchReport,
    },
    Failed {
        generation: u64,
        error: String,
    },
}

struct SearchRequest {
    generation: u64,
    root: PathBuf,
    query: String,
    options: ListingOptions,
    limits: SearchLimits,
    cancel: Arc<AtomicBool>,
    callback: Box<dyn Fn(SearchEvent) + Send + 'static>,
}

enum SearchMessage {
    Run(SearchRequest),
    Shutdown,
}

pub(in crate::desktop) struct SearchScheduler {
    pending: Arc<(Mutex<Option<SearchMessage>>, Condvar)>,
    active_cancel: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    stop: Arc<AtomicBool>,
}

impl SearchScheduler {
    pub(in crate::desktop) fn new() -> Result<Self, ()> {
        let pending = Arc::new((Mutex::new(None), Condvar::new()));
        let active_cancel = Arc::new(Mutex::new(None::<Arc<AtomicBool>>));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_pending = Arc::clone(&pending);
        let worker_stop = Arc::clone(&stop);
        let worker_active_cancel = Arc::clone(&active_cancel);
        thread::Builder::new()
            .name("rovex-search-worker".to_owned())
            .spawn(move || {
                loop {
                    let message = {
                        let (lock, condition) = &*worker_pending;
                        let Ok(message) = lock.lock() else {
                            break;
                        };
                        let mut message = match condition.wait_while(message, |message| {
                            message.is_none() && !worker_stop.load(Ordering::Acquire)
                        }) {
                            Ok(message) => message,
                            Err(_) => break,
                        };
                        if message.is_none() && worker_stop.load(Ordering::Acquire) {
                            break;
                        }
                        message.take()
                    };
                    let Some(message) = message else {
                        continue;
                    };
                    let SearchMessage::Run(request) = message else {
                        break;
                    };
                    let generation = request.generation;
                    let cancel = Arc::clone(&request.cancel);
                    let callback = request.callback;
                    let result = search_by_name(
                        &request.root,
                        &request.query,
                        request.options,
                        request.limits,
                        &cancel,
                        |update| {
                            callback(SearchEvent::Update { generation, update });
                        },
                    );
                    if let Ok(mut active) = worker_active_cancel.lock()
                        && active
                            .as_ref()
                            .is_some_and(|current| Arc::ptr_eq(current, &request.cancel))
                    {
                        *active = None;
                    }
                    match result {
                        Ok(report) => callback(SearchEvent::Finished { generation, report }),
                        Err(error) => callback(SearchEvent::Failed {
                            generation,
                            error: error.to_string(),
                        }),
                    }
                }
            })
            .map_err(|_| ())?;

        Ok(Self {
            pending,
            active_cancel,
            stop,
        })
    }

    pub(in crate::desktop) fn start<F>(
        &self,
        generation: u64,
        root: PathBuf,
        query: String,
        options: ListingOptions,
        limits: SearchLimits,
        callback: F,
    ) -> Result<(), ()>
    where
        F: Fn(SearchEvent) + Send + 'static,
    {
        let cancel = Arc::new(AtomicBool::new(false));
        if let Ok(active) = self.active_cancel.lock()
            && let Some(active) = active.as_ref()
        {
            active.store(true, Ordering::Release);
        }
        if let Ok(mut active) = self.active_cancel.lock() {
            *active = Some(Arc::clone(&cancel));
        } else {
            return Err(());
        }
        let (lock, condition) = &*self.pending;
        let Ok(mut pending) = lock.lock() else {
            return Err(());
        };
        *pending = Some(SearchMessage::Run(SearchRequest {
            generation,
            root,
            query,
            options,
            limits,
            cancel,
            callback: Box::new(callback),
        }));
        condition.notify_one();
        Ok(())
    }

    pub(in crate::desktop) fn cancel(&self) {
        if let Ok(active) = self.active_cancel.lock()
            && let Some(active) = active.as_ref()
        {
            active.store(true, Ordering::Release);
        }
    }
}

impl Drop for SearchScheduler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Ok(active) = self.active_cancel.lock()
            && let Some(active) = active.as_ref()
        {
            active.store(true, Ordering::Release);
        }
        let (lock, condition) = &*self.pending;
        if let Ok(mut pending) = lock.lock() {
            *pending = Some(SearchMessage::Shutdown);
            condition.notify_one();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("rovex-search-scheduler-{stamp}"));
        fs::create_dir_all(&root).expect("root");
        root
    }

    #[test]
    fn emits_finished_event_from_worker() {
        let root = temp_root();
        fs::write(root.join("result.txt"), b"result").expect("file");
        let scheduler = SearchScheduler::new().expect("worker");
        let (sender, receiver) = mpsc::channel();
        scheduler
            .start(
                7,
                root.clone(),
                "result".to_owned(),
                ListingOptions::default(),
                SearchLimits::default(),
                move |event| sender.send(event).expect("event"),
            )
            .expect("start");
        let mut finished = None;
        for _ in 0..4 {
            if let SearchEvent::Finished { report, .. } = receiver
                .recv_timeout(Duration::from_secs(2))
                .expect("worker event")
            {
                finished = Some(report);
                break;
            }
        }
        assert_eq!(finished.expect("finished").matches, 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn starting_new_request_cancels_previous_request() {
        let root = temp_root();
        for index in 0..500 {
            fs::write(root.join(format!("entry-{index:04}.txt")), b"x").expect("file");
        }
        let scheduler = SearchScheduler::new().expect("worker");
        let (sender, receiver) = mpsc::channel();
        let first_sender = sender.clone();
        scheduler
            .start(
                1,
                root.clone(),
                "entry".to_owned(),
                ListingOptions::default(),
                SearchLimits::default(),
                move |event| {
                    let _ = first_sender.send(event);
                },
            )
            .expect("first start");
        scheduler
            .start(
                2,
                root.clone(),
                "entry-0499".to_owned(),
                ListingOptions::default(),
                SearchLimits::default(),
                move |event| {
                    let _ = sender.send(event);
                },
            )
            .expect("second start");
        let mut second_finished = false;
        for _ in 0..100 {
            if let Ok(SearchEvent::Finished {
                generation: 2,
                report,
            }) = receiver.recv_timeout(Duration::from_millis(100))
            {
                assert_eq!(report.matches, 1);
                second_finished = true;
                break;
            }
        }
        assert!(second_finished);
        fs::remove_dir_all(root).expect("cleanup");
    }
}
