use crate::converters::{ConversionError, ConversionKind, ConversionStage, convert_file};
use crate::operations::{
    CopyProgress, OperationError, copy_file_atomic_with_progress, delete_entry, rename_entry,
};
use slint::SharedString;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Sender},
};
use std::thread;

use super::MainWindow;
use super::state::{self, LoadedRow, SharedRows, SharedSelection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OperationKind {
    Copy,
    Move,
    Rename,
    Delete,
}

#[derive(Debug, Clone)]
pub(super) struct ConversionRequest {
    pub(super) kind: ConversionKind,
    pub(super) sources: Vec<PathBuf>,
    pub(super) refresh_path: PathBuf,
}

#[derive(Debug)]
struct ConversionOutcome {
    kind: ConversionKind,
    completed: usize,
    failed: Vec<String>,
    cancelled: bool,
}

impl ConversionOutcome {
    fn message(&self) -> String {
        let mut message = if self.cancelled {
            format!(
                "Conversão cancelada: {} item(ns) concluído(s), {} falha(s).",
                self.completed,
                self.failed.len()
            )
        } else if self.failed.is_empty() {
            format!(
                "Conversão para {} concluída: {} item(ns).",
                self.kind.label(),
                self.completed
            )
        } else {
            format!(
                "Conversão para {} concluída parcialmente: {} item(ns) concluído(s), {} falha(s).",
                self.kind.label(),
                self.completed,
                self.failed.len()
            )
        };
        for failure in self.failed.iter().take(3) {
            message.push('\n');
            message.push_str(failure);
        }
        if self.failed.len() > 3 {
            message.push_str(&format!("\n… e mais {} falha(s).", self.failed.len() - 3));
        }
        message
    }

    fn status(&self) -> String {
        if self.cancelled {
            "Conversão cancelada; a pasta será atualizada.".to_owned()
        } else if self.failed.is_empty() {
            "Conversão concluída; a pasta será atualizada.".to_owned()
        } else {
            "Conversão concluída parcialmente; verifique o resultado.".to_owned()
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct OperationRequest {
    pub(super) kind: OperationKind,
    pub(super) sources: Vec<PathBuf>,
    pub(super) destination_directory: Option<PathBuf>,
    pub(super) rename_name: Option<String>,
    pub(super) refresh_path: PathBuf,
}

#[derive(Debug)]
struct OperationOutcome {
    kind: OperationKind,
    completed: usize,
    failed: Vec<String>,
    cancelled: bool,
}

impl OperationOutcome {
    fn message(&self) -> String {
        let action = match self.kind {
            OperationKind::Copy => "cópia",
            OperationKind::Move => "movimentação",
            OperationKind::Rename => "renomeação",
            OperationKind::Delete => "exclusão",
        };
        let mut message = if self.cancelled {
            format!(
                "Operação cancelada: {} item(ns) concluído(s), {} falha(s).",
                self.completed,
                self.failed.len()
            )
        } else if self.failed.is_empty() {
            format!("{} concluída: {} item(ns).", action, self.completed)
        } else {
            format!(
                "{} concluída parcialmente: {} item(ns) concluído(s), {} falha(s).",
                action,
                self.completed,
                self.failed.len()
            )
        };
        for failure in self.failed.iter().take(3) {
            message.push('\n');
            message.push_str(failure);
        }
        if self.failed.len() > 3 {
            message.push_str(&format!("\n… e mais {} falha(s).", self.failed.len() - 3));
        }
        message
    }

    fn status(&self) -> String {
        if self.cancelled {
            "Operação cancelada; a pasta será atualizada.".to_owned()
        } else if self.failed.is_empty() {
            "Operação concluída; a pasta será atualizada.".to_owned()
        } else {
            "Operação concluída parcialmente; verifique o resultado.".to_owned()
        }
    }
}

#[derive(Debug)]
struct OperationUpdate {
    completed_items: usize,
    total_items: usize,
    current_bytes: u64,
    current_total_bytes: u64,
    explicit_percent: Option<u8>,
    label: String,
}

pub(super) fn operation_label(source: &Path) -> String {
    source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.display().to_string())
}

fn operation_destination(
    request: &OperationRequest,
    source: &Path,
) -> Result<PathBuf, OperationError> {
    let directory = request
        .destination_directory
        .as_ref()
        .ok_or(OperationError::Validation(
            crate::security::ValidationError::EmptyPath,
        ))?;
    let file_name = source.file_name().ok_or(OperationError::Validation(
        crate::security::ValidationError::EmptyPath,
    ))?;
    Ok(directory.join(file_name))
}

fn emit_item_progress<F>(
    emit: &mut F,
    index: usize,
    total_items: usize,
    label: &str,
    current_bytes: u64,
    current_total_bytes: u64,
) where
    F: FnMut(OperationUpdate),
{
    emit(OperationUpdate {
        completed_items: index,
        total_items,
        current_bytes,
        current_total_bytes,
        explicit_percent: None,
        label: label.to_owned(),
    });
}

fn emit_stage_progress<F>(
    emit: &mut F,
    index: usize,
    total_items: usize,
    label: &str,
    stage: ConversionStage,
) where
    F: FnMut(OperationUpdate),
{
    emit(OperationUpdate {
        completed_items: index,
        total_items,
        current_bytes: 0,
        current_total_bytes: 0,
        explicit_percent: Some(stage.percent()),
        label: label.to_owned(),
    });
}

fn execute_operation<F>(
    request: &OperationRequest,
    cancel: &AtomicBool,
    mut emit: F,
) -> OperationOutcome
where
    F: FnMut(OperationUpdate),
{
    let total_items = request.sources.len();
    let mut completed = 0;
    let mut failed = Vec::new();

    for (index, source) in request.sources.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            return OperationOutcome {
                kind: request.kind,
                completed,
                failed,
                cancelled: true,
            };
        }
        let label = operation_label(source);
        let result = match request.kind {
            OperationKind::Copy => {
                let destination = operation_destination(request, source);
                match destination {
                    Ok(destination) => copy_file_atomic_with_progress(
                        source,
                        &destination,
                        cancel,
                        |CopyProgress {
                             bytes_copied,
                             total_bytes,
                         }| {
                            emit_item_progress(
                                &mut emit,
                                index,
                                total_items,
                                &label,
                                bytes_copied,
                                total_bytes,
                            );
                        },
                    )
                    .map(|_| ()),
                    Err(error) => Err(error),
                }
            }
            OperationKind::Move => {
                let destination = operation_destination(request, source);
                match destination {
                    Ok(destination) => match rename_entry(source, &destination) {
                        Ok(()) => Ok(()),
                        Err(error) if error.is_cross_device() => {
                            let copy_result = copy_file_atomic_with_progress(
                                source,
                                &destination,
                                cancel,
                                |CopyProgress {
                                     bytes_copied,
                                     total_bytes,
                                 }| {
                                    emit_item_progress(
                                        &mut emit,
                                        index,
                                        total_items,
                                        &label,
                                        bytes_copied,
                                        total_bytes,
                                    );
                                },
                            );
                            match copy_result {
                                Ok(_) if cancel.load(Ordering::Acquire) => {
                                    return OperationOutcome {
                                        kind: request.kind,
                                        completed,
                                        failed: vec![format!(
                                            "{label}: cópia concluída, origem preservada porque o cancelamento foi solicitado"
                                        )],
                                        cancelled: true,
                                    };
                                }
                                Ok(_) => delete_entry(source),
                                Err(error) => Err(error),
                            }
                        }
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                }
            }
            OperationKind::Rename => {
                let Some(name) = request.rename_name.as_deref() else {
                    return OperationOutcome {
                        kind: request.kind,
                        completed,
                        failed: vec![format!("{label}: novo nome ausente")],
                        cancelled: false,
                    };
                };
                let Some(parent) = source.parent() else {
                    return OperationOutcome {
                        kind: request.kind,
                        completed,
                        failed: vec![format!("{label}: diretório pai ausente")],
                        cancelled: false,
                    };
                };
                rename_entry(source, &parent.join(name))
            }
            OperationKind::Delete => delete_entry(source),
        };

        match result {
            Ok(()) => {
                completed += 1;
                emit_item_progress(&mut emit, index + 1, total_items, &label, 0, 0);
            }
            Err(OperationError::Cancelled) => {
                return OperationOutcome {
                    kind: request.kind,
                    completed,
                    failed,
                    cancelled: true,
                };
            }
            Err(error) => failed.push(format!("{label}: {error}")),
        }
    }

    OperationOutcome {
        kind: request.kind,
        completed,
        failed,
        cancelled: false,
    }
}

#[derive(Debug)]
enum OperationMessage {
    Run(OperationRequest, Arc<AtomicBool>),
    Shutdown,
}

pub(super) struct OperationScheduler {
    sender: Sender<OperationMessage>,
    cancel: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
}

impl OperationScheduler {
    pub(super) fn new(
        ui_weak: slint::Weak<MainWindow>,
        load_scheduler: Option<Arc<LoadScheduler>>,
    ) -> Result<Self, ()> {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(false));
        let worker_busy = Arc::clone(&busy);
        let worker_ui = ui_weak.clone();
        thread::Builder::new()
            .name("rovex-operation-worker".to_owned())
            .spawn(move || {
                while let Ok(message) = receiver.recv() {
                    let OperationMessage::Run(request, request_cancel) = message else {
                        break;
                    };
                    let progress_ui = worker_ui.clone();
                    let mut last_percentage = -1_i32;
                    let outcome = execute_operation(&request, &request_cancel, |update| {
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

    pub(super) fn start(&self, request: OperationRequest) -> Result<(), OperationRequest> {
        if self.busy.swap(true, Ordering::AcqRel) {
            return Err(request);
        }
        self.cancel.store(false, Ordering::Release);
        let message = OperationMessage::Run(request, Arc::clone(&self.cancel));
        match self.sender.send(message) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.busy.store(false, Ordering::Release);
                match error.0 {
                    OperationMessage::Run(request, _) => Err(request),
                    OperationMessage::Shutdown => unreachable!("shutdown não é enviado por start"),
                }
            }
        }
    }

    pub(super) fn cancel(&self) {
        if self.busy.load(Ordering::Acquire) {
            self.cancel.store(true, Ordering::Release);
        }
    }
}

impl Drop for OperationScheduler {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        let _ = self.sender.send(OperationMessage::Shutdown);
    }
}

fn execute_conversion<F>(
    request: &ConversionRequest,
    cancel: &AtomicBool,
    mut emit: F,
) -> ConversionOutcome
where
    F: FnMut(OperationUpdate),
{
    let total_items = request.sources.len();
    let mut completed = 0;
    let mut failed = Vec::new();

    for (index, source) in request.sources.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            return ConversionOutcome {
                kind: request.kind,
                completed,
                failed,
                cancelled: true,
            };
        }
        let label = operation_label(source);
        let result = convert_file(source, request.kind, cancel, |stage| {
            emit_stage_progress(&mut emit, index, total_items, &label, stage);
        });
        match result {
            Ok(_) => {
                completed += 1;
                emit_item_progress(&mut emit, index + 1, total_items, &label, 0, 0);
            }
            Err(ConversionError::Cancelled) => {
                return ConversionOutcome {
                    kind: request.kind,
                    completed,
                    failed,
                    cancelled: true,
                };
            }
            Err(error) => failed.push(format!("{label}: {error}")),
        }
    }

    ConversionOutcome {
        kind: request.kind,
        completed,
        failed,
        cancelled: false,
    }
}

#[derive(Debug)]
enum ConversionMessage {
    Run(ConversionRequest, Arc<AtomicBool>),
    Shutdown,
}

pub(super) struct ConversionScheduler {
    sender: Sender<ConversionMessage>,
    cancel: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
}

impl ConversionScheduler {
    pub(super) fn new(
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

    pub(super) fn start(&self, request: ConversionRequest) -> Result<(), ConversionRequest> {
        if self.busy.swap(true, Ordering::AcqRel) {
            return Err(request);
        }
        self.cancel.store(false, Ordering::Release);
        let message = ConversionMessage::Run(request, Arc::clone(&self.cancel));
        match self.sender.send(message) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.busy.store(false, Ordering::Release);
                match error.0 {
                    ConversionMessage::Run(request, _) => Err(request),
                    ConversionMessage::Shutdown => unreachable!("shutdown não é enviado por start"),
                }
            }
        }
    }

    pub(super) fn cancel(&self) {
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

struct FilterRequest {
    generation: u64,
    query: String,
}

pub(super) struct FilterScheduler {
    pending: Arc<(Mutex<Option<FilterRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
}

impl FilterScheduler {
    pub(super) fn new(
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

    pub(super) fn schedule(&self, generation: u64, query: String) -> Result<(), ()> {
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

struct LoadRequest {
    generation: u64,
    path: PathBuf,
}

pub(super) struct LoadScheduler {
    pending: Arc<(Mutex<Option<LoadRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
    load_generation: Arc<AtomicU64>,
    filter_generation: Arc<AtomicU64>,
}

impl LoadScheduler {
    pub(super) fn new(
        ui_weak: slint::Weak<MainWindow>,
        directory_rows: SharedRows,
        selection: SharedSelection,
        filter_generation: Arc<AtomicU64>,
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
                    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                        if ui_load_generation.load(Ordering::Acquire) != request.generation {
                            return;
                        }
                        ui.set_current_path(SharedString::from(
                            loaded.path.to_string_lossy().to_string(),
                        ));
                        ui_filter_generation.fetch_add(1, Ordering::AcqRel);
                        ui.set_filter_text(SharedString::default());
                        let empty_state = if loaded.is_error {
                            ""
                        } else {
                            state::empty_state_text(loaded.rows.len(), loaded.rows.len(), "")
                        };
                        ui.set_empty_state_text(SharedString::from(empty_state));
                        ui.set_focused_row_index(-1);
                        let Ok(mut selection_state) = ui_selection.lock() else {
                            ui.set_status_text("Falha interna ao limpar a seleção".into());
                            return;
                        };
                        selection_state.clear();
                        ui.set_selection_count(0);
                        let snapshot: Arc<[LoadedRow]> = Arc::from(loaded.rows);
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

    pub(super) fn schedule(&self, path: PathBuf) -> Result<(), ()> {
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

pub(super) fn start_load(
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
