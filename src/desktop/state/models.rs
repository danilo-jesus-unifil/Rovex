use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(in crate::desktop) struct LoadedRow {
    pub(in crate::desktop) key: String,
    pub(in crate::desktop) path: PathBuf,
    pub(in crate::desktop) name: String,
    pub(in crate::desktop) kind: String,
    pub(in crate::desktop) icon: String,
    pub(in crate::desktop) details: String,
    pub(in crate::desktop) size: Option<u64>,
    pub(in crate::desktop) modified: Option<std::time::SystemTime>,
    pub(in crate::desktop) created: Option<std::time::SystemTime>,
    pub(in crate::desktop) accessed: Option<std::time::SystemTime>,
    pub(in crate::desktop) is_directory: bool,
}

pub(in crate::desktop) struct LoadedDirectory {
    pub(in crate::desktop) path: PathBuf,
    pub(in crate::desktop) rows: Vec<LoadedRow>,
    pub(in crate::desktop) status: String,
    pub(in crate::desktop) is_error: bool,
}

pub(in crate::desktop) type SharedRows = Arc<Mutex<Arc<[LoadedRow]>>>;
pub(in crate::desktop) type SharedSelection = Arc<Mutex<SelectionState>>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::desktop) struct SelectionState {
    pub(in crate::desktop) selected: BTreeSet<String>,
    pub(in crate::desktop) anchor: Option<String>,
}

impl SelectionState {
    pub(in crate::desktop) fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    pub(in crate::desktop) fn select_all<I>(&mut self, keys: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.selected = keys.into_iter().collect();
        self.anchor = None;
    }

    pub(in crate::desktop) fn click(
        &mut self,
        key: &str,
        visible_keys: &[String],
        control: bool,
        shift: bool,
    ) {
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

    pub(in crate::desktop) fn count(&self) -> usize {
        self.selected.len()
    }
}
