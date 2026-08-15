use crate::{DirectoryEntry, EntryKind, FileSystem};
use slint::{Model, ModelRc, SharedString, VecModel};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;

slint::include_modules!();

#[derive(Clone)]
struct LoadedRow {
    key: String,
    name: String,
    kind: String,
    details: String,
    is_directory: bool,
}

struct LoadedDirectory {
    path: PathBuf,
    rows: Vec<LoadedRow>,
    status: String,
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

fn row_from_entry(entry: &DirectoryEntry) -> LoadedRow {
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
        key: entry.path.to_string_lossy().into_owned(),
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
            rows: entries.iter().map(row_from_entry).collect(),
            status: format!("{} itens", entries.len()),
        },
        Err(error) => LoadedDirectory {
            path,
            rows: Vec::new(),
            status: format!("Não foi possível listar a pasta: {error}"),
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
}

impl FilterScheduler {
    fn new(
        ui_weak: slint::Weak<MainWindow>,
        directory_rows: SharedRows,
        selection: SharedSelection,
        filter_generation: Arc<AtomicU64>,
    ) -> Result<Self, ()> {
        let pending = Arc::new((Mutex::new(None::<FilterRequest>), std::sync::Condvar::new()));
        let worker_pending = Arc::clone(&pending);
        thread::Builder::new()
            .name("rovex-filter-worker".to_owned())
            .spawn(move || loop {
                let request = {
                    let (lock, condition) = &*worker_pending;
                    let Ok(pending) = lock.lock() else {
                        break;
                    };
                    let mut pending =
                        match condition.wait_while(pending, |request| request.is_none()) {
                            Ok(pending) => pending,
                            Err(_) => break,
                        };
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
                        (Some(filtered), status)
                    }
                    None => (None, "Falha interna ao ler a listagem".to_owned()),
                };
                let ui_filter_generation = Arc::clone(&filter_generation);
                let ui_selection = Arc::clone(&selection);
                let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                    if ui_filter_generation.load(Ordering::Acquire) != request.generation {
                        return;
                    }
                    let (filtered, status) = result;
                    let Ok(selection) = ui_selection.lock() else {
                        ui.set_status_text("Falha interna ao ler a seleção".into());
                        return;
                    };
                    ui.set_status_text(SharedString::from(status));
                    if let Some(filtered) = filtered {
                        if !set_rows(&ui, filtered, &selection) {
                            ui.set_status_text("Falha interna ao atualizar a lista".into());
                        }
                    }
                });
            })
            .map_err(|_| ())?;

        Ok(Self { pending })
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

fn start_load(
    ui_weak: slint::Weak<MainWindow>,
    path: PathBuf,
    load_generation: &Arc<AtomicU64>,
    filter_generation: &Arc<AtomicU64>,
    directory_rows: &SharedRows,
    selection: &SharedSelection,
) {
    let generation = load_generation.fetch_add(1, Ordering::AcqRel) + 1;
    filter_generation.fetch_add(1, Ordering::AcqRel);
    let load_generation = Arc::clone(load_generation);
    let filter_generation = Arc::clone(filter_generation);
    let directory_rows = Arc::clone(directory_rows);
    let selection = Arc::clone(selection);
    let failure_ui_weak = ui_weak.clone();
    let worker = thread::Builder::new()
        .name("rovex-filesystem-loader".to_owned())
        .spawn(move || {
            let loaded = load_directory(path);
            let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                if load_generation.load(Ordering::Acquire) != generation {
                    return;
                }
                ui.set_current_path(SharedString::from(
                    loaded.path.to_string_lossy().to_string(),
                ));
                filter_generation.fetch_add(1, Ordering::AcqRel);
                ui.set_filter_text(SharedString::default());
                let Ok(mut selection_state) = selection.lock() else {
                    ui.set_status_text("Falha interna ao limpar a seleção".into());
                    return;
                };
                selection_state.clear();
                let snapshot: Arc<[LoadedRow]> = Arc::from(loaded.rows);
                let Ok(mut rows) = directory_rows.lock() else {
                    ui.set_status_text("Falha interna ao armazenar a listagem".into());
                    return;
                };
                *rows = Arc::clone(&snapshot);
                ui.set_status_text(SharedString::from(loaded.status));
                if !set_rows(&ui, snapshot.as_ref().to_vec(), &selection_state) {
                    ui.set_status_text("Falha interna ao atualizar a lista".into());
                }
            });
        });

    if worker.is_err() {
        let _ = failure_ui_weak.upgrade_in_event_loop(|ui| {
            ui.set_status_text("Falha ao iniciar o carregamento".into());
        });
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
    let load_generation = Arc::new(AtomicU64::new(0));
    let filter_generation = Arc::new(AtomicU64::new(0));
    let filter_scheduler = FilterScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&selection),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        ui.on_refresh_requested(move || {
            let path = history.borrow().current.clone();
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        ui.on_navigate_to(move |text| {
            let path = PathBuf::from(text.to_string());
            let changed = history.borrow_mut().visit(path.clone());
            if changed {
                update_history_controls(&ui_weak, &history.borrow());
            }
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let locations = locations.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
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
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        ui.on_back_requested(move || {
            let Some(path) = history.borrow_mut().go_back() else {
                return;
            };
            update_history_controls(&ui_weak, &history.borrow());
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        ui.on_forward_requested(move || {
            let Some(path) = history.borrow_mut().go_forward() else {
                return;
            };
            update_history_controls(&ui_weak, &history.borrow());
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        ui.on_navigate_up(move || {
            let current = history.borrow().current.clone();
            let Some(parent) = parent_directory(&current) else {
                return;
            };
            history.borrow_mut().visit(parent.clone());
            update_history_controls(&ui_weak, &history.borrow());
            start_load(
                ui_weak.clone(),
                parent,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let history = history.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
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
            let next = history.borrow().current.join(row.name.as_str());
            history.borrow_mut().visit(next.clone());
            update_history_controls(&ui_weak, &history.borrow());
            start_load(
                ui_weak.clone(),
                next,
                &load_generation,
                &filter_generation,
                &directory_rows,
                &selection,
            );
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
                    ui.set_status_text(SharedString::from(selection_status(&state)));
                }
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let filter_generation = Arc::clone(&filter_generation);
        let filter_scheduler = filter_scheduler.clone();
        ui.on_filter_changed(move |text| {
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
    start_load(
        ui.as_weak(),
        initial_path,
        &load_generation,
        &filter_generation,
        &directory_rows,
        &selection,
    );
    ui.run()
}

#[cfg(test)]
mod tests {
    use super::{
        default_locations, filter_rows, filter_status, format_size, load_directory,
        parent_directory, LoadedRow, NavigationHistory, SelectionState,
    };
    use std::path::Path;

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
        assert!(locations
            .iter()
            .any(|location| location.path == Path::new(".")));
        assert!(locations.iter().all(|location| !location.label.is_empty()));
        assert!(locations.iter().all(|location| location.path.is_dir()));
    }

    #[test]
    fn filtro_localiza_nome_sem_varrer_subpastas() {
        let rows = vec![
            LoadedRow {
                key: "foto".to_owned(),
                name: "Foto.JPG".to_owned(),
                kind: "[FILE]".to_owned(),
                details: "4 KB".to_owned(),
                is_directory: false,
            },
            LoadedRow {
                key: "projetos".to_owned(),
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

    #[test]
    fn erro_de_diretorio_inexistente_vira_status_controlado() {
        let path = std::env::temp_dir().join("rovex-path-that-does-not-exist");
        let loaded = load_directory(path);
        assert!(loaded.rows.is_empty());
        assert!(loaded
            .status
            .starts_with("Não foi possível listar a pasta:"));
    }
}
