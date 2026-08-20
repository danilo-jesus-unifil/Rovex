use super::*;
use crate::filesystem::{DirectoryEntry, ListingOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

fn collect(root: &Path, query: &str, limits: SearchLimits) -> (SearchReport, Vec<DirectoryEntry>) {
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
fn symlink_root_is_rejected_without_following_target() {
    use std::os::unix::fs::symlink;
    let root = temp_root("symlink-root");
    let outside = temp_root("symlink-root-outside");
    fs::write(outside.join("secret.txt"), b"secret").expect("write outside");
    let linked_root = root.join("linked-root");
    symlink(&outside, &linked_root).expect("create root symlink");

    let cancel = Arc::new(AtomicBool::new(false));
    let error = search_by_name(
        &linked_root,
        "secret",
        ListingOptions::default(),
        SearchLimits::default(),
        &cancel,
        |_| {},
    )
    .expect_err("symlink root must be rejected");
    assert!(matches!(error, SearchError::RootRedirected(_)));
    fs::remove_dir_all(root).expect("cleanup root");
    fs::remove_dir_all(outside).expect("cleanup outside");
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
