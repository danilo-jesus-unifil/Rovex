use crate::{DirectoryEntry, EntryKind, FileSystem};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Sender},
};
use std::thread;

use crate::converters::{ConversionError, ConversionKind, ConversionStage, convert_file};
use crate::operations::{
    CopyProgress, OperationError, copy_file_atomic_with_progress, delete_entry, rename_entry,
};

slint::include_modules!();

#[derive(Clone)]
struct LoadedRow {
    key: String,
    path: PathBuf,
    name: String,
    kind: String,
    details: String,
    is_directory: bool,
}

struct LoadedDirectory {
    path: PathBuf,
    rows: Vec<LoadedRow>,
    status: String,
    is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocationEntry {
    label: String,
    path: PathBuf,
}

fn add_location(locations: &mut Vec<LocationEntry>, label: &str, path: PathBuf) {
    if !path.is_dir() || locations.iter().any(|location| location.path == path) {
        return;
    }
    locations.push(LocationEntry {
        label: label.to_owned(),
        path,
    });
}

fn default_locations(initial_path: &Path) -> Vec<LocationEntry> {
    let mut locations = Vec::new();
    if let Some(home) = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
    {
        add_location(&mut locations, "Início", home.clone());
        add_location(&mut locations, "Área de Trabalho", home.join("Desktop"));
        add_location(&mut locations, "Documentos", home.join("Documents"));
        add_location(&mut locations, "Downloads", home.join("Downloads"));
    }
    add_location(&mut locations, "Pasta atual", initial_path.to_path_buf());
    #[cfg(unix)]
    add_location(&mut locations, "Sistema", PathBuf::from("/"));
    locations
}

type SharedRows = Arc<Mutex<Arc<[LoadedRow]>>>;
type SharedSelection = Arc<Mutex<SelectionState>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SelectionState {
    selected: BTreeSet<String>,
    anchor: Option<String>,
}

impl SelectionState {
    fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    fn select_all<I>(&mut self, keys: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.selected = keys.into_iter().collect();
        self.anchor = None;
    }

    fn click(&mut self, key: &str, visible_keys: &[String], control: bool, shift: bool) {
        if shift {
            let anchor_index = self.anchor.as_deref().and_then(|anchor| {
                visible_keys
                    .iter()
                    .position(|candidate| candidate == anchor)
            });
            let current_index = visible_keys.iter().position(|candidate| candidate == key);
            if let (Some(anchor_index), Some(current_index)) = (anchor_index, current_index) {
                if !control {
                    self.selected.clear();
                }
                let (start, end) = if anchor_index <= current_index {
                    (anchor_index, current_index)
                } else {
                    (current_index, anchor_index)
                };
                self.selected
                    .extend(visible_keys[start..=end].iter().cloned());
            } else {
                self.selected.clear();
                self.selected.insert(key.to_owned());
            }
        } else if control {
            if !self.selected.insert(key.to_owned()) {
                self.selected.remove(key);
            }
        } else {
            self.selected.clear();
            self.selected.insert(key.to_owned());
        }
        self.anchor = Some(key.to_owned());
    }

    fn count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationKind {
    Copy,
    Move,
    Rename,
    Delete,
}

#[derive(Debug, Clone)]
struct ConversionRequest {
    kind: ConversionKind,
    sources: Vec<PathBuf>,
    refresh_path: PathBuf,
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
struct OperationRequest {
    kind: OperationKind,
    sources: Vec<PathBuf>,
    destination_directory: Option<PathBuf>,
    rename_name: Option<String>,
    refresh_path: PathBuf,
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

fn operation_label(source: &Path) -> String {
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

struct OperationScheduler {
    sender: Sender<OperationMessage>,
    cancel: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
}

impl OperationScheduler {
    fn new(
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

    fn start(&self, request: OperationRequest) -> Result<(), OperationRequest> {
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

    fn cancel(&self) {
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

struct ConversionScheduler {
    sender: Sender<ConversionMessage>,
    cancel: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
}

impl ConversionScheduler {
    fn new(
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

    fn start(&self, request: ConversionRequest) -> Result<(), ConversionRequest> {
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

    fn cancel(&self) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct NavigationHistory {
    current: PathBuf,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
}

impl NavigationHistory {
    fn new(current: PathBuf) -> Self {
        Self {
            current,
            back: Vec::new(),
            forward: Vec::new(),
        }
    }

    fn visit(&mut self, path: PathBuf) -> bool {
        if self.current == path {
            return false;
        }
        self.back.push(self.current.clone());
        self.current = path;
        self.forward.clear();
        true
    }

    fn go_back(&mut self) -> Option<PathBuf> {
        let path = self.back.pop()?;
        self.forward.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    fn go_forward(&mut self) -> Option<PathBuf> {
        let path = self.forward.pop()?;
        self.back.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}

fn filter_rows(rows: &[LoadedRow], query: &str) -> Vec<LoadedRow> {
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        return rows.to_vec();
    }

    rows.iter()
        .filter(|row| row.name.to_lowercase().contains(&normalized_query))
        .cloned()
        .collect()
}

fn empty_state_text(total: usize, visible: usize, query: &str) -> &'static str {
    if total == 0 {
        "Esta pasta está vazia."
    } else if visible == 0 && !query.trim().is_empty() {
        "Nenhum item corresponde ao filtro."
    } else {
        ""
    }
}

fn filter_status(total: usize, visible: usize, query: &str) -> String {
    if query.trim().is_empty() {
        return format!("{total} itens");
    }
    if visible == 0 {
        return format!(
            "Nenhum item corresponde a ‘{}’ ({total} itens na pasta)",
            query.trim()
        );
    }
    format!("{visible} de {total} itens")
}

fn row_from_entry(entry: &DirectoryEntry, index: usize) -> LoadedRow {
    let (kind, is_directory) = match entry.kind {
        EntryKind::Directory => ("[DIR]", true),
        EntryKind::File => ("[FILE]", false),
        EntryKind::Symlink => ("[LINK]", false),
        EntryKind::Other => ("[OTHER]", false),
    };

    let details = entry
        .size
        .map(format_size)
        .unwrap_or_else(|| "—".to_owned());

    LoadedRow {
        key: format!("{}#{index}", entry.path.to_string_lossy()),
        path: entry.path.clone(),
        name: entry.display_name(),
        kind: kind.to_owned(),
        details,
        is_directory,
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn load_directory(path: PathBuf) -> LoadedDirectory {
    match FileSystem.list_directory(&path) {
        Ok(entries) => LoadedDirectory {
            path,
            rows: entries
                .iter()
                .enumerate()
                .map(|(index, entry)| row_from_entry(entry, index))
                .collect(),
            status: format!("{} itens", entries.len()),
            is_error: false,
        },
        Err(error) => LoadedDirectory {
            path,
            rows: Vec::new(),
            status: format!("Não foi possível listar a pasta: {error}"),
            is_error: true,
        },
    }
}

fn parent_directory(path: &Path) -> Option<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
}

fn set_rows(ui: &MainWindow, rows: Vec<LoadedRow>, selection: &SelectionState) -> bool {
    let entries = ui.get_entries();
    let Some(model) = entries.as_any().downcast_ref::<VecModel<FileRow>>() else {
        return false;
    };

    model.set_vec(
        rows.into_iter()
            .map(|row| FileRow {
                selected: selection.selected.contains(&row.key),
                key: SharedString::from(row.key),
                name: SharedString::from(row.name),
                kind: SharedString::from(row.kind),
                details: SharedString::from(row.details),
                is_directory: row.is_directory,
            })
            .collect::<Vec<_>>(),
    );
    true
}

fn update_selection_visuals(ui: &MainWindow, selection: &SelectionState) -> bool {
    let entries = ui.get_entries();
    let Some(model) = entries.as_any().downcast_ref::<VecModel<FileRow>>() else {
        return false;
    };

    for index in 0..model.row_count() {
        let Some(mut row) = model.row_data(index) else {
            continue;
        };
        let selected = selection.selected.contains(row.key.as_str());
        if row.selected != selected {
            row.selected = selected;
            model.set_row_data(index, row);
        }
    }
    true
}

fn selection_status(selection: &SelectionState) -> String {
    match selection.count() {
        0 => String::new(),
        1 => "1 item selecionado".to_owned(),
        count => format!("{count} itens selecionados"),
    }
}

struct FilterRequest {
    generation: u64,
    query: String,
}

struct FilterScheduler {
    pending: Arc<(Mutex<Option<FilterRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
}

impl FilterScheduler {
    fn new(
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
                            let filtered = filter_rows(rows.as_ref(), &request.query);
                            let status = filter_status(rows.len(), filtered.len(), &request.query);
                            let empty_state =
                                empty_state_text(rows.len(), filtered.len(), &request.query);
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
                            && !set_rows(&ui, filtered, &selection)
                        {
                            ui.set_status_text("Falha interna ao atualizar a lista".into());
                        }
                    });
                }
            })
            .map_err(|_| ())?;

        Ok(Self { pending, stop })
    }

    fn schedule(&self, generation: u64, query: String) -> Result<(), ()> {
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

struct LoadScheduler {
    pending: Arc<(Mutex<Option<LoadRequest>>, std::sync::Condvar)>,
    stop: Arc<AtomicBool>,
    load_generation: Arc<AtomicU64>,
    filter_generation: Arc<AtomicU64>,
}

impl LoadScheduler {
    fn new(
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
                    let loaded = load_directory(request.path);
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
                            empty_state_text(loaded.rows.len(), loaded.rows.len(), "")
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
                        if !set_rows(&ui, snapshot.as_ref().to_vec(), &selection_state) {
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

    fn schedule(&self, path: PathBuf) -> Result<(), ()> {
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

fn start_load(
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

fn selected_paths(rows: &SharedRows, selection: &SharedSelection) -> Vec<PathBuf> {
    let Ok(selection) = selection.lock() else {
        return Vec::new();
    };
    let Ok(rows) = rows.lock() else {
        return Vec::new();
    };
    rows.iter()
        .filter(|row| selection.selected.contains(&row.key))
        .map(|row| row.path.clone())
        .collect()
}

fn validate_rename_name(name: &str) -> Result<String, &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("o novo nome não pode ser vazio");
    }
    if trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\0')
    {
        return Err("o novo nome deve ser um único nome de arquivo");
    }
    Ok(trimmed.to_owned())
}

fn show_selected_operation_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<OperationRequest>>>,
    directory_rows: &SharedRows,
    selection: &SharedSelection,
    history: &Rc<std::cell::RefCell<NavigationHistory>>,
    kind: OperationKind,
) {
    let sources = selected_paths(directory_rows, selection);
    if sources.is_empty() {
        return;
    }
    let (title, message, input, needs_input) = match kind {
        OperationKind::Copy => (
            "Copiar itens",
            format!(
                "Confirme a cópia de {} item(ns). Informe um diretório de destino absoluto. Destinos existentes não serão sobrescritos.",
                sources.len()
            ),
            String::new(),
            true,
        ),
        OperationKind::Move => (
            "Mover itens",
            format!(
                "Confirme a movimentação de {} item(ns). Informe um diretório de destino absoluto. Destinos existentes não serão sobrescritos.",
                sources.len()
            ),
            String::new(),
            true,
        ),
        OperationKind::Rename => {
            let Some(source) = sources.first() else {
                return;
            };
            let name = operation_label(source);
            (
                "Renomear item",
                "Informe um único nome novo. Separadores de caminho, ponto e ponto-ponto não são permitidos.".to_owned(),
                name,
                true,
            )
        }
        OperationKind::Delete => (
            "Excluir itens",
            format!(
                "Confirme a exclusão de {} item(ns). A operação não é recursiva: diretórios não vazios serão preservados.",
                sources.len()
            ),
            String::new(),
            false,
        ),
    };
    let request = OperationRequest {
        kind,
        sources,
        destination_directory: None,
        rename_name: if kind == OperationKind::Rename {
            Some(input.clone())
        } else {
            None
        },
        refresh_path: history.borrow().current.clone(),
    };
    show_operation_dialog(
        ui_weak,
        pending,
        request,
        title,
        &message,
        &input,
        needs_input,
    );
}

fn show_conversion_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<ConversionRequest>>>,
    directory_rows: &SharedRows,
    selection: &SharedSelection,
    history: &Rc<std::cell::RefCell<NavigationHistory>>,
    kind: ConversionKind,
) {
    let sources = selected_paths(directory_rows, selection);
    if sources.is_empty() {
        return;
    }
    let request = ConversionRequest {
        kind,
        sources: sources.clone(),
        refresh_path: history.borrow().current.clone(),
    };
    *pending.borrow_mut() = Some(request);
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_context_menu_visible(false);
        ui.set_context_menu_can_jxl(false);
        ui.set_context_menu_can_opus(false);
        ui.set_context_menu_can_png(false);
        ui.set_context_menu_can_flac(false);
        ui.set_operation_dialog_title("Converter arquivos".into());
        ui.set_operation_dialog_message(SharedString::from(format!(
            "Confirme a conversão de {} item(ns) para {}. A saída será criada no mesmo diretório e nunca sobrescreverá um arquivo existente.",
            sources.len(),
            kind.label()
        )));
        ui.set_operation_dialog_input(SharedString::default());
        ui.set_operation_needs_input(false);
        ui.set_operation_close_only(false);
        ui.set_operation_busy(false);
        ui.set_operation_progress(0);
        ui.set_operation_progress_text(SharedString::default());
        ui.set_operation_dialog_visible(true);
    }
}

fn show_operation_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<OperationRequest>>>,
    request: OperationRequest,
    title: &str,
    message: &str,
    input: &str,
    needs_input: bool,
) {
    *pending.borrow_mut() = Some(request);
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_operation_dialog_title(SharedString::from(title));
        ui.set_operation_dialog_message(SharedString::from(message));
        ui.set_operation_dialog_input(SharedString::from(input));
        ui.set_operation_needs_input(needs_input);
        ui.set_operation_close_only(false);
        ui.set_operation_busy(false);
        ui.set_operation_progress(0);
        ui.set_operation_progress_text(SharedString::default());
        ui.set_context_menu_visible(false);
        ui.set_context_menu_can_jxl(false);
        ui.set_context_menu_can_opus(false);
        ui.set_context_menu_can_png(false);
        ui.set_context_menu_can_flac(false);
        ui.set_operation_dialog_visible(true);
    }
}

fn update_history_controls(ui_weak: &slint::Weak<MainWindow>, history: &NavigationHistory) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_can_go_back(history.can_go_back());
        ui.set_can_go_forward(history.can_go_forward());
    }
}

pub fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let entries = Rc::new(VecModel::<FileRow>::default());
    ui.set_entries(ModelRc::from(entries.clone()));
    let locations = Rc::new(VecModel::<LocationRow>::default());
    ui.set_locations(ModelRc::from(locations.clone()));

    let initial_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    locations.set_vec(
        default_locations(&initial_path)
            .into_iter()
            .map(|location| LocationRow {
                label: SharedString::from(location.label),
                path: SharedString::from(location.path.to_string_lossy().to_string()),
            })
            .collect::<Vec<_>>(),
    );
    ui.set_current_path(SharedString::from(
        initial_path.to_string_lossy().to_string(),
    ));
    ui.set_status_text("Carregando…".into());

    let ui_weak = ui.as_weak();
    let history = Rc::new(std::cell::RefCell::new(NavigationHistory::new(
        initial_path.clone(),
    )));
    let directory_rows: SharedRows = Arc::new(Mutex::new(Arc::from(Vec::<LoadedRow>::new())));
    let selection: SharedSelection = Arc::new(Mutex::new(SelectionState::default()));
    let filter_generation = Arc::new(AtomicU64::new(0));
    let load_scheduler = LoadScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&selection),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();
    let filter_scheduler = FilterScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&selection),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();
    let pending_operation = Rc::new(std::cell::RefCell::new(None::<OperationRequest>));
    let pending_conversion = Rc::new(std::cell::RefCell::new(None::<ConversionRequest>));
    let operation_scheduler = OperationScheduler::new(ui_weak.clone(), load_scheduler.clone())
        .map(Arc::new)
        .ok();
    let conversion_scheduler = ConversionScheduler::new(ui_weak.clone(), load_scheduler.clone())
        .map(Arc::new)
        .ok();

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_refresh_requested(move || {
            let path = history.borrow().current.clone();
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_to(move |text| {
            let path = PathBuf::from(text.to_string());
            let changed = history.borrow_mut().visit(path.clone());
            if changed {
                update_history_controls(&ui_weak, &history.borrow());
            }
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let locations = locations.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_to_location(move |index| {
            if index < 0 {
                return;
            }
            let Some(location) = locations.row_data(index as usize) else {
                return;
            };
            let path = PathBuf::from(location.path.to_string());
            history.borrow_mut().visit(path.clone());
            update_history_controls(&ui_weak, &history.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_back_requested(move || {
            let Some(path) = history.borrow_mut().go_back() else {
                return;
            };
            update_history_controls(&ui_weak, &history.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_forward_requested(move || {
            let Some(path) = history.borrow_mut().go_forward() else {
                return;
            };
            update_history_controls(&ui_weak, &history.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_up(move || {
            let current = history.borrow().current.clone();
            let Some(parent) = parent_directory(&current) else {
                return;
            };
            history.borrow_mut().visit(parent.clone());
            update_history_controls(&ui_weak, &history.borrow());
            start_load(&ui_weak, parent, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let history = history.clone();
        let load_scheduler = load_scheduler.clone();
        let directory_rows = Arc::clone(&directory_rows);
        ui.on_activate(move |index| {
            if index < 0 {
                return;
            }
            let Some(row) = entries.row_data(index as usize) else {
                return;
            };
            if !row.is_directory {
                return;
            }
            let Ok(rows) = directory_rows.lock() else {
                return;
            };
            let Some(next) = rows
                .iter()
                .find(|loaded_row| loaded_row.key == row.key.as_str())
                .map(|loaded_row| loaded_row.path.clone())
            else {
                return;
            };
            history.borrow_mut().visit(next.clone());
            update_history_controls(&ui_weak, &history.borrow());
            start_load(&ui_weak, next, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
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
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
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
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
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
            let is_regular_file = row.kind == "[FILE]";
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
            ui.set_context_menu_visible(true);
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_copy_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_move_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_rename_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_delete_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Delete,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_copy_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_move_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_rename_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_delete_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &history,
                OperationKind::Delete,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_convert_jxl_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &history,
                ConversionKind::JpegXl,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_convert_opus_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &history,
                ConversionKind::Opus,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_convert_png_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &history,
                ConversionKind::Png,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let history = history.clone();
        ui.on_context_menu_convert_flac_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &history,
                ConversionKind::Flac,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let pending_conversion = pending_conversion.clone();
        let operation_scheduler = operation_scheduler.clone();
        let conversion_scheduler = conversion_scheduler.clone();
        ui.on_operation_confirmed(move || {
            let conversion_request = pending_conversion.borrow_mut().take();
            if let Some(request) = conversion_request {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };
                let Some(scheduler) = conversion_scheduler.as_ref() else {
                    ui.set_operation_dialog_message(
                        "O worker de conversão está indisponível.".into(),
                    );
                    *pending_conversion.borrow_mut() = Some(request);
                    return;
                };
                if let Err(request) = scheduler.start(request) {
                    ui.set_operation_dialog_message(
                        "Já existe uma conversão em andamento; aguarde o resultado.".into(),
                    );
                    *pending_conversion.borrow_mut() = Some(request);
                    return;
                }
                ui.set_operation_busy(true);
                ui.set_operation_close_only(false);
                ui.set_operation_needs_input(false);
                ui.set_operation_progress(0);
                ui.set_operation_progress_text("Preparando conversão…".into());
                return;
            }

            let Some(mut request) = pending_operation.borrow_mut().take() else {
                return;
            };
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match request.kind {
                OperationKind::Copy | OperationKind::Move => {
                    let input = ui.get_operation_dialog_input().to_string();
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        ui.set_operation_dialog_message("Informe um diretório de destino.".into());
                        *pending_operation.borrow_mut() = Some(request);
                        return;
                    }
                    request.destination_directory = Some(PathBuf::from(trimmed));
                }
                OperationKind::Rename => {
                    let input = ui.get_operation_dialog_input().to_string();
                    match validate_rename_name(&input) {
                        Ok(name) => request.rename_name = Some(name),
                        Err(error) => {
                            ui.set_operation_dialog_message(SharedString::from(error));
                            *pending_operation.borrow_mut() = Some(request);
                            return;
                        }
                    }
                }
                OperationKind::Delete => {}
            }
            let Some(scheduler) = operation_scheduler.as_ref() else {
                ui.set_operation_dialog_message("O worker de operações está indisponível.".into());
                *pending_operation.borrow_mut() = Some(request);
                return;
            };
            if let Err(request) = scheduler.start(request) {
                ui.set_operation_dialog_message(
                    "Já existe uma operação em andamento; aguarde o resultado.".into(),
                );
                *pending_operation.borrow_mut() = Some(request);
                return;
            }
            ui.set_operation_busy(true);
            ui.set_operation_close_only(false);
            ui.set_operation_needs_input(false);
            ui.set_operation_progress(0);
            ui.set_operation_progress_text("Preparando…".into());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let operation_scheduler = operation_scheduler.clone();
        let conversion_scheduler = conversion_scheduler.clone();
        ui.on_operation_cancelled(move || {
            if let Some(scheduler) = operation_scheduler.as_ref() {
                scheduler.cancel();
            }
            if let Some(scheduler) = conversion_scheduler.as_ref() {
                scheduler.cancel();
            }
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_operation_progress_text("Cancelamento solicitado…".into());
                ui.set_operation_dialog_message(
                    "A tarefa será interrompida no próximo ponto seguro; o resultado parcial será verificado.".into(),
                );
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let pending_conversion = pending_conversion.clone();
        ui.on_operation_dismissed(move || {
            if let Some(ui) = ui_weak.upgrade()
                && !ui.get_operation_busy()
            {
                ui.set_operation_dialog_visible(false);
                *pending_operation.borrow_mut() = None;
                *pending_conversion.borrow_mut() = None;
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        ui.on_context_menu_dismissed(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
        });
    }

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

    update_history_controls(&ui_weak, &history.borrow());
    start_load(&ui_weak, initial_path, load_scheduler.as_ref());
    ui.run()
}

#[cfg(test)]
mod tests {
    use super::{
        LoadedRow, NavigationHistory, SelectionState, default_locations, empty_state_text,
        filter_rows, filter_status, format_size, load_directory, parent_directory,
        validate_rename_name,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn selecao_ctrl_shift_e_ctrl_a_mantem_intervalos_reais() {
        let keys = vec![
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        let mut selection = SelectionState::default();

        selection.click("b", &keys, false, false);
        assert_eq!(selection.count(), 1);
        assert!(selection.selected.contains("b"));

        selection.click("d", &keys, true, false);
        assert_eq!(selection.count(), 2);
        assert!(selection.selected.contains("b"));
        assert!(selection.selected.contains("d"));

        selection.click("a", &keys, false, true);
        assert_eq!(selection.count(), 4);
        assert!(selection.selected.contains("a"));
        assert!(selection.selected.contains("b"));
        assert!(selection.selected.contains("c"));
        assert!(selection.selected.contains("d"));

        selection.select_all(keys.clone());
        assert_eq!(selection.count(), keys.len());
    }

    #[test]
    fn renomear_recusa_traversal_e_preserva_nome_unicode() {
        assert!(validate_rename_name("").is_err());
        assert!(validate_rename_name("..").is_err());
        assert!(validate_rename_name("pasta/arquivo.txt").is_err());
        assert!(validate_rename_name("pasta\\arquivo.txt").is_err());
        assert_eq!(
            validate_rename_name(" relatório final.txt"),
            Ok("relatório final.txt".to_owned())
        );
    }

    #[test]
    fn historico_navega_para_tras_e_para_frente_e_limpa_futuro() {
        let mut history = NavigationHistory::new(Path::new("/inicio").to_path_buf());
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        assert!(history.visit(Path::new("/projetos").to_path_buf()));
        assert!(history.can_go_back());
        assert_eq!(history.go_back(), Some(Path::new("/inicio").to_path_buf()));
        assert!(history.can_go_forward());
        assert_eq!(
            history.go_forward(),
            Some(Path::new("/projetos").to_path_buf())
        );

        assert!(history.visit(Path::new("/documentos").to_path_buf()));
        assert!(!history.can_go_forward());
    }

    #[test]
    fn locais_padrao_so_incluem_diretorios_existentes() {
        let locations = default_locations(Path::new("."));
        assert!(
            locations
                .iter()
                .any(|location| location.path == Path::new("."))
        );
        assert!(locations.iter().all(|location| !location.label.is_empty()));
        assert!(locations.iter().all(|location| location.path.is_dir()));
    }

    #[test]
    fn filtro_localiza_nome_sem_varrer_subpastas() {
        let rows = vec![
            LoadedRow {
                key: "foto".to_owned(),
                path: PathBuf::from("foto"),
                name: "Foto.JPG".to_owned(),
                kind: "[FILE]".to_owned(),
                details: "4 KB".to_owned(),
                is_directory: false,
            },
            LoadedRow {
                key: "projetos".to_owned(),
                path: PathBuf::from("projetos"),
                name: "Projetos".to_owned(),
                kind: "[DIR]".to_owned(),
                details: "—".to_owned(),
                is_directory: true,
            },
        ];

        let filtered = filter_rows(&rows, "jpg");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Foto.JPG");
        assert_eq!(filter_rows(&rows, "   ").len(), 2);
        assert_eq!(filter_status(2, 1, "jpg"), "1 de 2 itens");
    }

    #[test]
    #[ignore = "benchmark manual de performance"]
    fn benchmark_filtro_100k() {
        use std::time::Instant;

        let rows = (0..100_000)
            .map(|index| LoadedRow {
                key: format!("/tmp/file-{index:05}.txt"),
                path: PathBuf::from(format!("/tmp/file-{index:05}.txt")),
                name: format!("file-{index:05}.txt"),
                kind: "[FILE]".to_owned(),
                details: "1 B".to_owned(),
                is_directory: false,
            })
            .collect::<Vec<_>>();
        let started = Instant::now();
        let filtered = filter_rows(&rows, "99999");
        let elapsed = started.elapsed();
        eprintln!(
            "benchmark_filter_100k elapsed_ms={} matches={}",
            elapsed.as_secs_f64() * 1000.0,
            filtered.len()
        );
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn estados_vazios_diferenciam_pasta_e_filtro() {
        assert_eq!(empty_state_text(0, 0, ""), "Esta pasta está vazia.");
        assert_eq!(
            empty_state_text(4, 0, "pdf"),
            "Nenhum item corresponde ao filtro."
        );
        assert_eq!(empty_state_text(4, 4, "pdf"), "");
    }

    #[test]
    fn filtro_sem_resultado_exibe_estado_vazio_controlado() {
        let status = filter_status(4, 0, "pdf");
        assert_eq!(status, "Nenhum item corresponde a ‘pdf’ (4 itens na pasta)");
    }

    #[test]
    fn formata_tamanho_sem_overflow_visual() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn encontra_pasta_pai_sem_transformar_raiz_em_pasta_vazia() {
        assert_eq!(
            parent_directory(Path::new("/tmp/rovex")),
            Some(Path::new("/tmp").to_path_buf())
        );
        assert_eq!(parent_directory(Path::new("/")), None);
    }

    #[test]
    fn carrega_diretorio_real_sem_fingir_sucesso() {
        let path = std::env::current_dir().expect("o diretório atual deve existir");
        let loaded = load_directory(path);
        assert!(!loaded.rows.is_empty());
        assert!(loaded.status.ends_with("itens"));
    }

    #[cfg(unix)]
    #[test]
    fn preserva_caminhos_de_nomes_invalidos_sem_colidir_chaves() {
        use std::ffi::OsString;
        use std::fs;
        use std::os::unix::ffi::OsStringExt;

        let root =
            std::env::temp_dir().join(format!("rovex-invalid-name-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).expect("a pasta de teste deve ser criada");
        let first = root.join(OsString::from_vec(vec![0xff, b'.', b't', b'x', b't']));
        let second = root.join(OsString::from_vec(vec![0xfe, b'.', b't', b'x', b't']));
        fs::write(&first, b"a").expect("o primeiro arquivo deve ser criado");
        fs::write(&second, b"b").expect("o segundo arquivo deve ser criado");

        let loaded = load_directory(root.clone());
        assert_eq!(loaded.rows.len(), 2);
        assert_ne!(loaded.rows[0].key, loaded.rows[1].key);
        assert_ne!(loaded.rows[0].path, loaded.rows[1].path);
        fs::remove_dir_all(root).expect("a pasta de teste deve ser removida");
    }

    #[test]
    fn erro_de_diretorio_inexistente_vira_status_controlado() {
        let path = std::env::temp_dir().join("rovex-path-that-does-not-exist");
        let loaded = load_directory(path);
        assert!(loaded.rows.is_empty());
        assert!(loaded.is_error);
        assert!(
            loaded
                .status
                .starts_with("Não foi possível listar a pasta:")
        );
    }
}
