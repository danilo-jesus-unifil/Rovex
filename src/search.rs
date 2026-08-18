use crate::filesystem::{DirectoryEntry, FileSystemError, ListingOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const BATCH_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchLimit {
    Results,
    Directories,
    Entries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStatus {
    Completed,
    Cancelled,
    Limited(SearchLimit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchLimits {
    pub max_results: usize,
    pub max_visited_directories: usize,
    pub max_visited_entries: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_results: 10_000,
            max_visited_directories: 100_000,
            max_visited_entries: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchUpdate {
    pub batch: Vec<DirectoryEntry>,
    pub visited_directories: usize,
    pub visited_entries: usize,
    pub ignored_entries: usize,
    pub status: Option<SearchStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchReport {
    pub status: SearchStatus,
    pub matches: usize,
    pub visited_directories: usize,
    pub visited_entries: usize,
    pub ignored_entries: usize,
}

#[derive(Debug)]
pub enum SearchError {
    EmptyQuery,
    RelativeRoot(PathBuf),
    Root(FileSystemError),
    RootNotDirectory(PathBuf),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyQuery => write!(formatter, "a pesquisa não pode ser vazia"),
            Self::RelativeRoot(path) => write!(
                formatter,
                "a raiz da pesquisa deve ser absoluta: {}",
                path.display()
            ),
            Self::Root(error) => write!(
                formatter,
                "não foi possível abrir a raiz da pesquisa: {error}"
            ),
            Self::RootNotDirectory(path) => {
                write!(
                    formatter,
                    "a raiz da pesquisa não é uma pasta: {}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for SearchError {}

pub fn search_by_name<F>(
    root: &Path,
    query: &str,
    options: ListingOptions,
    limits: SearchLimits,
    cancel: &Arc<AtomicBool>,
    mut emit: F,
) -> Result<SearchReport, SearchError>
where
    F: FnMut(SearchUpdate),
{
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(SearchError::EmptyQuery);
    }
    if root.as_os_str().is_empty() || root.is_relative() {
        return Err(SearchError::RelativeRoot(root.to_path_buf()));
    }
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|error| SearchError::Root(FileSystemError::from_io("abrir raiz", root, error)))?;
    if !root_metadata.is_dir() {
        return Err(SearchError::RootNotDirectory(root.to_path_buf()));
    }

    let mut pending = vec![root.to_path_buf()];
    let mut matches = 0;
    let mut visited_directories = 0;
    let mut visited_entries = 0;
    let mut ignored_entries = 0;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    while let Some(directory) = pending.pop() {
        if cancel.load(Ordering::Acquire) {
            emit_update(
                &mut emit,
                &mut batch,
                visited_directories,
                visited_entries,
                ignored_entries,
                Some(SearchStatus::Cancelled),
            );
            return Ok(SearchReport {
                status: SearchStatus::Cancelled,
                matches,
                visited_directories,
                visited_entries,
                ignored_entries,
            });
        }
        if visited_directories >= limits.max_visited_directories {
            emit_update(
                &mut emit,
                &mut batch,
                visited_directories,
                visited_entries,
                ignored_entries,
                Some(SearchStatus::Limited(SearchLimit::Directories)),
            );
            return Ok(report(
                SearchStatus::Limited(SearchLimit::Directories),
                matches,
                visited_directories,
                visited_entries,
                ignored_entries,
            ));
        }
        visited_directories += 1;

        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                ignored_entries += 1;
                continue;
            }
        };
        let mut child_directories = Vec::new();
        for entry in entries {
            if cancel.load(Ordering::Acquire) {
                emit_update(
                    &mut emit,
                    &mut batch,
                    visited_directories,
                    visited_entries,
                    ignored_entries,
                    Some(SearchStatus::Cancelled),
                );
                return Ok(report(
                    SearchStatus::Cancelled,
                    matches,
                    visited_directories,
                    visited_entries,
                    ignored_entries,
                ));
            }
            if visited_entries >= limits.max_visited_entries {
                emit_update(
                    &mut emit,
                    &mut batch,
                    visited_directories,
                    visited_entries,
                    ignored_entries,
                    Some(SearchStatus::Limited(SearchLimit::Entries)),
                );
                return Ok(report(
                    SearchStatus::Limited(SearchLimit::Entries),
                    matches,
                    visited_directories,
                    visited_entries,
                    ignored_entries,
                ));
            }
            visited_entries += 1;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    ignored_entries += 1;
                    continue;
                }
            };
            let path = entry.path();
            let directory_entry = match DirectoryEntry::from_path(path.clone(), entry.file_name()) {
                Ok(entry) => entry,
                Err(_) => {
                    ignored_entries += 1;
                    continue;
                }
            };
            if (!options.show_hidden && directory_entry.is_hidden)
                || (!options.show_system && directory_entry.is_system)
            {
                continue;
            }
            if directory_entry
                .display_name()
                .to_lowercase()
                .contains(&query)
            {
                if matches >= limits.max_results {
                    emit_update(
                        &mut emit,
                        &mut batch,
                        visited_directories,
                        visited_entries,
                        ignored_entries,
                        Some(SearchStatus::Limited(SearchLimit::Results)),
                    );
                    return Ok(report(
                        SearchStatus::Limited(SearchLimit::Results),
                        matches,
                        visited_directories,
                        visited_entries,
                        ignored_entries,
                    ));
                }
                matches += 1;
                batch.push(directory_entry.clone());
                if batch.len() >= BATCH_SIZE {
                    emit_update(
                        &mut emit,
                        &mut batch,
                        visited_directories,
                        visited_entries,
                        ignored_entries,
                        None,
                    );
                }
            }
            if directory_entry.kind == crate::filesystem::EntryKind::Directory
                && !is_reparse_point(&path)
            {
                child_directories.push(path);
            }
        }
        child_directories.sort_by_cached_key(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_lowercase())
                .unwrap_or_default()
        });
        pending.extend(child_directories.into_iter().rev());
    }

    emit_update(
        &mut emit,
        &mut batch,
        visited_directories,
        visited_entries,
        ignored_entries,
        Some(SearchStatus::Completed),
    );
    Ok(report(
        SearchStatus::Completed,
        matches,
        visited_directories,
        visited_entries,
        ignored_entries,
    ))
}

fn report(
    status: SearchStatus,
    matches: usize,
    visited_directories: usize,
    visited_entries: usize,
    ignored_entries: usize,
) -> SearchReport {
    SearchReport {
        status,
        matches,
        visited_directories,
        visited_entries,
        ignored_entries,
    }
}

fn emit_update<F>(
    emit: &mut F,
    batch: &mut Vec<DirectoryEntry>,
    visited_directories: usize,
    visited_entries: usize,
    ignored_entries: usize,
    status: Option<SearchStatus>,
) where
    F: FnMut(SearchUpdate),
{
    if batch.is_empty() && status.is_none() {
        return;
    }
    emit(SearchUpdate {
        batch: std::mem::take(batch),
        visited_directories,
        visited_entries,
        ignored_entries,
        status,
    });
}

fn is_reparse_point(path: &Path) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return true;
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rovex-search-{label}-{stamp}"));
        fs::create_dir_all(&path).expect("create root");
        path
    }

    fn collect(
        root: &Path,
        query: &str,
        limits: SearchLimits,
    ) -> (SearchReport, Vec<DirectoryEntry>) {
        let mut results = Vec::new();
        let cancel = Arc::new(AtomicBool::new(false));
        let report = search_by_name(
            root,
            query,
            ListingOptions::default(),
            limits,
            &cancel,
            |update| results.extend(update.batch),
        )
        .expect("search succeeds");
        (report, results)
    }

    #[test]
    fn finds_nested_names_in_deterministic_order() {
        let root = temp_root("nested");
        fs::create_dir(root.join("a")).expect("create a");
        fs::create_dir(root.join("b")).expect("create b");
        fs::write(root.join("b/match-two.txt"), b"2").expect("write two");
        fs::write(root.join("a/match-one.txt"), b"1").expect("write one");
        let (report, results) = collect(&root, "MATCH", SearchLimits::default());
        assert_eq!(report.status, SearchStatus::Completed);
        assert_eq!(
            results
                .iter()
                .map(|entry| entry.display_name())
                .collect::<Vec<_>>(),
            ["match-one.txt", "match-two.txt"]
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn cancellation_is_reported_without_false_completion() {
        let root = temp_root("cancel");
        fs::write(root.join("match.txt"), b"match").expect("write match");
        let cancel = Arc::new(AtomicBool::new(true));
        let mut statuses = Vec::new();
        let report = search_by_name(
            &root,
            "match",
            ListingOptions::default(),
            SearchLimits::default(),
            &cancel,
            |update| {
                assert!(update.batch.is_empty());
                statuses.push(update.status);
            },
        )
        .expect("cancel is a normal result");
        assert_eq!(statuses, [Some(SearchStatus::Cancelled)]);
        assert_eq!(report.status, SearchStatus::Cancelled);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn result_limit_is_explicitly_reported() {
        let root = temp_root("limit");
        fs::write(root.join("one.txt"), b"1").expect("write one");
        fs::write(root.join("two.txt"), b"2").expect("write two");
        let limits = SearchLimits {
            max_results: 1,
            ..SearchLimits::default()
        };
        let (report, results) = collect(&root, ".txt", limits);
        assert_eq!(report.status, SearchStatus::Limited(SearchLimit::Results));
        assert_eq!(results.len(), 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn relative_root_and_empty_query_are_rejected() {
        let cancel = Arc::new(AtomicBool::new(false));
        let error = search_by_name(
            Path::new("relative"),
            "x",
            ListingOptions::default(),
            SearchLimits::default(),
            &cancel,
            |_| {},
        )
        .expect_err("relative root");
        assert!(matches!(error, SearchError::RelativeRoot(_)));
        let error = search_by_name(
            Path::new("/tmp"),
            "  ",
            ListingOptions::default(),
            SearchLimits::default(),
            &cancel,
            |_| {},
        )
        .expect_err("empty query");
        assert!(matches!(error, SearchError::EmptyQuery));
    }

    #[test]
    fn hidden_entries_follow_listing_options() {
        let root = temp_root("hidden");
        fs::write(root.join(".secret.txt"), b"secret").expect("write hidden");
        let (_, hidden_by_default) = collect(&root, "secret", SearchLimits::default());
        assert!(hidden_by_default.is_empty());
        let cancel = Arc::new(AtomicBool::new(false));
        let mut shown = Vec::new();
        let report = search_by_name(
            &root,
            "secret",
            ListingOptions {
                show_hidden: true,
                show_system: false,
            },
            SearchLimits::default(),
            &cancel,
            |update| shown.extend(update.batch),
        )
        .expect("search hidden");
        assert_eq!(report.status, SearchStatus::Completed);
        assert_eq!(shown.len(), 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_directory_is_not_followed() {
        use std::os::unix::fs::symlink;
        let root = temp_root("symlink");
        let outside = temp_root("outside");
        fs::write(outside.join("secret.txt"), b"secret").expect("write outside");
        symlink(&outside, root.join("linked")).expect("create symlink");
        let (_, results) = collect(&root, "secret", SearchLimits::default());
        assert!(results.is_empty());
        fs::remove_dir_all(root).expect("cleanup root");
        fs::remove_dir_all(outside).expect("cleanup outside");
    }
}
