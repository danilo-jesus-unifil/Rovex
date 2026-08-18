use super::{LoadedDirectory, LoadedRow};
use crate::filesystem::{DirectoryEntry, EntryKind, FileSystem, ListingOptions};
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
    let (icon, default_kind, is_directory) = row_icon(&entry.display_name(), entry.kind);
    let kind = if entry.is_system {
        "Item do sistema"
    } else if entry.is_hidden {
        if is_directory {
            "Pasta oculta"
        } else {
            "Arquivo oculto"
        }
    } else {
        default_kind
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
        icon: icon.to_owned(),
        details,
        size: entry.size,
        modified: entry.modified,
        created: entry.created,
        accessed: entry.accessed,
        is_directory,
    }
}

pub(crate) fn format_timestamp(time: Option<std::time::SystemTime>) -> String {
    let Some(time) = time else {
        return "—".to_owned();
    };
    let Ok(seconds) = time.duration_since(std::time::UNIX_EPOCH) else {
        return "—".to_owned();
    };
    let total_minutes = seconds.as_secs() / 60;
    let minute = total_minutes % 60;
    let total_hours = total_minutes / 60;
    let hour = total_hours % 24;
    let days_since_epoch = total_hours / 24;
    let (year, month, day) = civil_date(days_since_epoch as i64);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

fn civil_date(days_since_epoch: i64) -> (i64, i64, i64) {
    let shifted = days_since_epoch + 719_468;
    let era = if shifted >= 0 {
        shifted / 146_097
    } else {
        (shifted - 146_096) / 146_097
    };
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
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

pub(in crate::desktop) fn load_directory(
    path: PathBuf,
    options: ListingOptions,
) -> LoadedDirectory {
    match FileSystem.list_directory_with_options(&path, options) {
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
