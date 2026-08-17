use super::{LoadedDirectory, LoadedRow};
use crate::filesystem::{DirectoryEntry, EntryKind, FileSystem};
use std::path::{Path, PathBuf};

pub(crate) fn row_icon(name: &str, kind: EntryKind) -> (&'static str, &'static str, bool) {
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

pub(crate) fn format_size(bytes: u64) -> String {
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

pub(in crate::desktop) fn load_directory(path: PathBuf) -> LoadedDirectory {
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

pub(in crate::desktop) fn parent_directory(path: &Path) -> Option<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
}
