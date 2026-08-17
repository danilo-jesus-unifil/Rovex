use crate::{DirectoryEntry, EntryKind, FileSystem};
use slint::{Model, SharedString, VecModel};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{FileRow, MainWindow, TabRow};

#[derive(Clone)]
pub(super) struct LoadedRow {
    pub(super) key: String,
    pub(super) path: PathBuf,
    name: String,
    kind: String,
    icon: String,
    details: String,
    is_directory: bool,
}

pub(super) struct LoadedDirectory {
    pub(super) path: PathBuf,
    pub(super) rows: Vec<LoadedRow>,
    pub(super) status: String,
    pub(super) is_error: bool,
}

pub(super) type SharedRows = Arc<Mutex<Arc<[LoadedRow]>>>;
pub(super) type SharedSelection = Arc<Mutex<SelectionState>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct SelectionState {
    selected: BTreeSet<String>,
    anchor: Option<String>,
}

impl SelectionState {
    pub(super) fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    pub(super) fn select_all<I>(&mut self, keys: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.selected = keys.into_iter().collect();
        self.anchor = None;
    }

    pub(super) fn click(&mut self, key: &str, visible_keys: &[String], control: bool, shift: bool) {
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

    pub(super) fn count(&self) -> usize {
        self.selected.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NavigationHistory {
    pub(super) current: PathBuf,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
}

impl NavigationHistory {
    pub(super) fn new(current: PathBuf) -> Self {
        Self {
            current,
            back: Vec::new(),
            forward: Vec::new(),
        }
    }

    pub(super) fn visit(&mut self, path: PathBuf) -> bool {
        if self.current == path {
            return false;
        }
        self.back.push(self.current.clone());
        self.current = path;
        self.forward.clear();
        true
    }

    pub(super) fn go_back(&mut self) -> Option<PathBuf> {
        let path = self.back.pop()?;
        self.forward.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    pub(super) fn go_forward(&mut self) -> Option<PathBuf> {
        let path = self.forward.pop()?;
        self.back.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    pub(super) fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub(super) fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}

#[derive(Debug)]
pub(super) struct TabManager {
    histories: Vec<NavigationHistory>,
    active: usize,
}

fn tab_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

impl TabManager {
    pub(super) fn new(initial_path: PathBuf) -> Self {
        Self {
            histories: vec![NavigationHistory::new(initial_path)],
            active: 0,
        }
    }

    pub(super) fn active(&self) -> &NavigationHistory {
        &self.histories[self.active]
    }

    pub(super) fn active_mut(&mut self) -> &mut NavigationHistory {
        &mut self.histories[self.active]
    }

    pub(super) fn select(&mut self, index: usize) -> bool {
        if index >= self.histories.len() || index == self.active {
            return false;
        }
        self.active = index;
        true
    }

    pub(super) fn new_tab(&mut self, path: PathBuf) {
        self.histories.push(NavigationHistory::new(path));
        self.active = self.histories.len() - 1;
    }

    pub(super) fn close(&mut self, index: usize) -> bool {
        if self.histories.len() <= 1 || index >= self.histories.len() {
            return false;
        }
        self.histories.remove(index);
        if self.active >= self.histories.len() {
            self.active = self.histories.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }
        true
    }

    pub(super) fn rows(&self) -> Vec<TabRow> {
        self.histories
            .iter()
            .enumerate()
            .map(|(index, history)| TabRow {
                label: SharedString::from(tab_label(&history.current)),
                path: SharedString::from(history.current.to_string_lossy().to_string()),
                active: index == self.active,
            })
            .collect()
    }
}

pub(super) fn filter_rows(rows: &[LoadedRow], query: &str) -> Vec<LoadedRow> {
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        return rows.to_vec();
    }

    rows.iter()
        .filter(|row| row.name.to_lowercase().contains(&normalized_query))
        .cloned()
        .collect()
}

pub(super) fn empty_state_text(total: usize, visible: usize, query: &str) -> &'static str {
    if total == 0 {
        "Esta pasta está vazia."
    } else if visible == 0 && !query.trim().is_empty() {
        "Nenhum item corresponde ao filtro."
    } else {
        ""
    }
}

pub(super) fn filter_status(total: usize, visible: usize, query: &str) -> String {
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

fn row_icon(name: &str, kind: EntryKind) -> (&'static str, &'static str, bool) {
    match kind {
        EntryKind::Directory => ("▰", "Pasta", true),
        EntryKind::Symlink => ("↗", "Link simbólico", false),
        EntryKind::Other => ("◇", "Outro tipo", false),
        EntryKind::File => {
            let extension = Path::new(name)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase());
            let icon = match extension.as_deref() {
                Some("html" | "htm") => "<>",
                Some("css") => "#",
                Some("rs") => "Rs",
                Some("py") => "Py",
                Some("ts" | "tsx") => "TS",
                Some("js" | "jsx") => "JS",
                Some("java") => "Jv",
                Some("json" | "toml" | "yaml" | "yml") => "{}",
                Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "jxl") => "◉",
                Some("mp3" | "wav" | "flac" | "opus" | "oga") => "♫",
                Some("mp4" | "mkv" | "webm" | "mov") => "▶",
                _ => "●",
            };
            (icon, "Arquivo", false)
        }
    }
}

fn row_from_entry(entry: &DirectoryEntry, index: usize) -> LoadedRow {
    let (icon, kind, is_directory) = row_icon(&entry.display_name(), entry.kind);

    let details = entry
        .size
        .map(format_size)
        .unwrap_or_else(|| "—".to_owned());

    LoadedRow {
        key: format!("{}#{index}", entry.path.to_string_lossy()),
        path: entry.path.clone(),
        name: entry.display_name(),
        kind: kind.to_owned(),
        icon: icon.to_owned(),
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

pub(super) fn load_directory(path: PathBuf) -> LoadedDirectory {
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

pub(super) fn parent_directory(path: &Path) -> Option<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
}

pub(super) fn set_rows(ui: &MainWindow, rows: Vec<LoadedRow>, selection: &SelectionState) -> bool {
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
                icon: SharedString::from(row.icon),
                details: SharedString::from(row.details),
                is_directory: row.is_directory,
            })
            .collect::<Vec<_>>(),
    );
    true
}

pub(super) fn update_selection_visuals(ui: &MainWindow, selection: &SelectionState) -> bool {
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

pub(super) fn selection_status(selection: &SelectionState) -> String {
    match selection.count() {
        0 => String::new(),
        1 => "1 item selecionado".to_owned(),
        count => format!("{count} itens selecionados"),
    }
}

pub(super) fn selected_paths(rows: &SharedRows, selection: &SharedSelection) -> Vec<PathBuf> {
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

pub(super) fn validate_rename_name(name: &str) -> Result<String, &'static str> {
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

#[cfg(test)]
mod tests {
    use super::super::locations::default_locations;
    use super::{
        LoadedRow, NavigationHistory, SelectionState, TabManager, empty_state_text, filter_rows,
        filter_status, format_size, load_directory, parent_directory, row_icon,
        validate_rename_name,
    };
    use crate::filesystem::EntryKind;

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
    fn abas_preservam_historicos_independentes_e_nao_fecham_a_ultima() {
        let mut tabs = TabManager::new(Path::new("/inicio").to_path_buf());
        tabs.active_mut()
            .visit(Path::new("/projetos").to_path_buf());
        tabs.new_tab(Path::new("/documentos").to_path_buf());
        assert_eq!(tabs.histories.len(), 2);
        assert_eq!(tabs.active().current, Path::new("/documentos"));
        assert!(tabs.select(0));
        assert_eq!(tabs.active().current, Path::new("/projetos"));
        assert!(tabs.active().can_go_back());
        assert!(tabs.close(1));
        assert!(!tabs.close(0));
        assert_eq!(tabs.histories.len(), 1);
    }

    #[test]
    fn icones_semanticos_diferenciam_pasta_arquivo_e_extensoes() {
        assert_eq!(
            row_icon("Fotos", EntryKind::Directory),
            ("▰", "Pasta", true)
        );
        assert_eq!(
            row_icon("imagem.png", EntryKind::File),
            ("◉", "Arquivo", false)
        );
        assert_eq!(
            row_icon("main.rs", EntryKind::File),
            ("Rs", "Arquivo", false)
        );
        assert_eq!(
            row_icon("atalho", EntryKind::Symlink),
            ("↗", "Link simbólico", false)
        );
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
                kind: "Arquivo".to_owned(),
                icon: "●".to_owned(),
                details: "4 KB".to_owned(),
                is_directory: false,
            },
            LoadedRow {
                key: "projetos".to_owned(),
                path: PathBuf::from("projetos"),
                name: "Projetos".to_owned(),
                kind: "Pasta".to_owned(),
                icon: "▰".to_owned(),
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
                kind: "Arquivo".to_owned(),
                icon: "●".to_owned(),
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
