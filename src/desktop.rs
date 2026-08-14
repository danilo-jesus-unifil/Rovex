use crate::{DirectoryEntry, EntryKind, FileSystem};
use slint::{Model, ModelRc, SharedString, VecModel};
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

type SharedRows = Arc<Mutex<Arc<[LoadedRow]>>>;

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

fn set_rows(ui: &MainWindow, rows: Vec<LoadedRow>) -> bool {
    let entries = ui.get_entries();
    let Some(model) = entries.as_any().downcast_ref::<VecModel<FileRow>>() else {
        return false;
    };

    model.set_vec(
        rows.into_iter()
            .map(|row| FileRow {
                name: SharedString::from(row.name),
                kind: SharedString::from(row.kind),
                details: SharedString::from(row.details),
                is_directory: row.is_directory,
            })
            .collect::<Vec<_>>(),
    );
    true
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
                let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                    if ui_filter_generation.load(Ordering::Acquire) != request.generation {
                        return;
                    }
                    let (filtered, status) = result;
                    ui.set_status_text(SharedString::from(status));
                    if let Some(filtered) = filtered {
                        if !set_rows(&ui, filtered) {
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
) {
    let generation = load_generation.fetch_add(1, Ordering::AcqRel) + 1;
    filter_generation.fetch_add(1, Ordering::AcqRel);
    let load_generation = Arc::clone(load_generation);
    let filter_generation = Arc::clone(filter_generation);
    let directory_rows = Arc::clone(directory_rows);
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
                let snapshot: Arc<[LoadedRow]> = Arc::from(loaded.rows);
                let Ok(mut rows) = directory_rows.lock() else {
                    ui.set_status_text("Falha interna ao armazenar a listagem".into());
                    return;
                };
                *rows = Arc::clone(&snapshot);
                ui.set_status_text(SharedString::from(loaded.status));
                if !set_rows(&ui, snapshot.as_ref().to_vec()) {
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

pub fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let entries = Rc::new(VecModel::<FileRow>::default());
    ui.set_entries(ModelRc::from(entries.clone()));

    let initial_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    ui.set_current_path(SharedString::from(
        initial_path.to_string_lossy().to_string(),
    ));
    ui.set_status_text("Carregando…".into());

    let ui_weak = ui.as_weak();
    let refresh_path = Rc::new(std::cell::RefCell::new(initial_path.clone()));
    let directory_rows: SharedRows = Arc::new(Mutex::new(Arc::from(Vec::<LoadedRow>::new())));
    let load_generation = Arc::new(AtomicU64::new(0));
    let filter_generation = Arc::new(AtomicU64::new(0));
    let filter_scheduler = FilterScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();

    {
        let ui_weak = ui_weak.clone();
        let refresh_path = refresh_path.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        ui.on_refresh_requested(move || {
            start_load(
                ui_weak.clone(),
                refresh_path.borrow().clone(),
                &load_generation,
                &filter_generation,
                &directory_rows,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let refresh_path = refresh_path.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        ui.on_navigate_to(move |text| {
            let path = PathBuf::from(text.to_string());
            *refresh_path.borrow_mut() = path.clone();
            start_load(
                ui_weak.clone(),
                path,
                &load_generation,
                &filter_generation,
                &directory_rows,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let refresh_path = refresh_path.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows = Arc::clone(&directory_rows);
        ui.on_navigate_up(move || {
            let current = refresh_path.borrow().clone();
            if let Some(parent) = parent_directory(&current) {
                *refresh_path.borrow_mut() = parent.clone();
                start_load(
                    ui_weak.clone(),
                    parent,
                    &load_generation,
                    &filter_generation,
                    &directory_rows,
                );
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let refresh_path = refresh_path.clone();
        let load_generation = Arc::clone(&load_generation);
        let filter_generation = Arc::clone(&filter_generation);
        let directory_rows_for_navigation = Arc::clone(&directory_rows);
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
            let next = refresh_path.borrow().join(row.name.as_str());
            *refresh_path.borrow_mut() = next.clone();
            start_load(
                ui_weak.clone(),
                next,
                &load_generation,
                &filter_generation,
                &directory_rows_for_navigation,
            );
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

    start_load(
        ui.as_weak(),
        initial_path,
        &load_generation,
        &filter_generation,
        &directory_rows,
    );
    ui.run()
}

#[cfg(test)]
mod tests {
    use super::{
        filter_rows, filter_status, format_size, load_directory, parent_directory, LoadedRow,
    };
    use std::path::Path;

    #[test]
    fn filtro_localiza_nome_sem_varrer_subpastas() {
        let rows = vec![
            LoadedRow {
                name: "Foto.JPG".to_owned(),
                kind: "[FILE]".to_owned(),
                details: "4 KB".to_owned(),
                is_directory: false,
            },
            LoadedRow {
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
