use super::super::MainWindow;
use super::conversion::execute_conversion;
use super::load_scheduler::LoadScheduler;
use super::types::ConversionRequest;
use slint::SharedString;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Sender},
};
use std::thread;

#[derive(Debug)]
enum ConversionMessage {
    Run(ConversionRequest, Arc<AtomicBool>),
    Shutdown,
}

pub(in crate::desktop) struct ConversionScheduler {
    sender: Sender<ConversionMessage>,
    cancel: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
}

impl ConversionScheduler {
    pub(in crate::desktop) fn new(
        ui_weak: slint::Weak<MainWindow>,
        load_scheduler: Option<Arc<LoadScheduler>>,
    ) -> Result<Self, ()> {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(false));
        let worker_busy = Arc::clone(&busy);
        let worker_ui = ui_weak.clone();
        thread::Builder::new()
            .name("rovex-conversion-worker".to_owned())
            .spawn(move || {
                while let Ok(message) = receiver.recv() {
                    let ConversionMessage::Run(request, request_cancel) = message else {
                        break;
                    };
                    let progress_ui = worker_ui.clone();
                    let mut last_percentage = -1_i32;
                    let outcome = execute_conversion(&request, &request_cancel, |update| {
                        let total = update.total_items.max(1) as f64;
                        let item_fraction = if update.current_total_bytes > 0 {
                            update.current_bytes as f64 / update.current_total_bytes as f64
                        } else {
                            0.0
                        };
                        let item_progress = update
                            .explicit_percent
                            .map(|percent| f64::from(percent) / 100.0)
                            .unwrap_or(item_fraction);
                        let percentage = (((update.completed_items as f64 + item_progress) / total)
                            * 100.0)
                            .round()
                            .clamp(0.0, 100.0) as i32;
                        if percentage == last_percentage {
                            return;
                        }
                        last_percentage = percentage;
                        let text = format!("{} — {}%", update.label, percentage);
                        let _ = progress_ui.upgrade_in_event_loop(move |ui| {
                            if ui.get_operation_busy() {
                                ui.set_operation_progress(percentage);
                                ui.set_operation_progress_text(SharedString::from(text));
                            }
                        });
                    });
                    worker_busy.store(false, Ordering::Release);
                    if let Some(load_scheduler) = load_scheduler.as_ref() {
                        let _ = load_scheduler.schedule(request.refresh_path.clone());
                    }
                    let message = outcome.message();
                    let status = outcome.status();
                    let progress = if outcome.cancelled || !outcome.failed.is_empty() {
                        0
                    } else {
                        100
                    };
                    let _ = worker_ui.upgrade_in_event_loop(move |ui| {
                        ui.set_operation_busy(false);
                        ui.set_operation_close_only(true);
                        ui.set_operation_needs_input(false);
                        ui.set_operation_progress(progress);
                        ui.set_operation_progress_text(SharedString::from("Resultado"));
                        ui.set_operation_dialog_message(SharedString::from(message));
                        ui.set_status_text(SharedString::from(status));
                    });
                }
            })
            .map_err(|_| ())?;

        Ok(Self {
            sender,
            cancel,
            busy,
        })
    }

    pub(in crate::desktop) fn start(
        &self,
        request: ConversionRequest,
    ) -> Result<(), ConversionRequest> {
        if self.busy.swap(true, Ordering::AcqRel) {
            return Err(request);
        }
        self.cancel.store(false, Ordering::Release);
        let retry_request = request.clone();
        let message = ConversionMessage::Run(request, Arc::clone(&self.cancel));
        match self.sender.send(message) {
            Ok(()) => Ok(()),
            Err(_) => {
                self.busy.store(false, Ordering::Release);
                Err(retry_request)
            }
        }
    }

    pub(in crate::desktop) fn cancel(&self) {
        if self.busy.load(Ordering::Acquire) {
            self.cancel.store(true, Ordering::Release);
        }
    }
}

impl Drop for ConversionScheduler {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        let _ = self.sender.send(ConversionMessage::Shutdown);
    }
}
