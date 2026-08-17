use super::super::TabRow;
use slint::SharedString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::desktop) struct NavigationHistory {
    pub(in crate::desktop) current: PathBuf,
    back: Vec<PathBuf>,
    forward: Vec<PathBuf>,
}

impl NavigationHistory {
    pub(in crate::desktop) fn new(current: PathBuf) -> Self {
        Self {
            current,
            back: Vec::new(),
            forward: Vec::new(),
        }
    }

    pub(in crate::desktop) fn visit(&mut self, path: PathBuf) -> bool {
        if self.current == path {
            return false;
        }
        self.back.push(self.current.clone());
        self.current = path;
        self.forward.clear();
        true
    }

    pub(in crate::desktop) fn go_back(&mut self) -> Option<PathBuf> {
        let path = self.back.pop()?;
        self.forward.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    pub(in crate::desktop) fn go_forward(&mut self) -> Option<PathBuf> {
        let path = self.forward.pop()?;
        self.back.push(self.current.clone());
        self.current = path.clone();
        Some(path)
    }

    pub(in crate::desktop) fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub(in crate::desktop) fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}

#[derive(Debug)]
pub(in crate::desktop) struct TabManager {
    pub(in crate::desktop) histories: Vec<NavigationHistory>,
    pub(in crate::desktop) active: usize,
}

fn tab_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

impl TabManager {
    pub(in crate::desktop) fn new(initial_path: PathBuf) -> Self {
        Self {
            histories: vec![NavigationHistory::new(initial_path)],
            active: 0,
        }
    }

    pub(in crate::desktop) fn active(&self) -> &NavigationHistory {
        &self.histories[self.active]
    }

    pub(in crate::desktop) fn active_mut(&mut self) -> &mut NavigationHistory {
        &mut self.histories[self.active]
    }

    pub(in crate::desktop) fn select(&mut self, index: usize) -> bool {
        if index >= self.histories.len() || index == self.active {
            return false;
        }
        self.active = index;
        true
    }

    pub(in crate::desktop) fn new_tab(&mut self, path: PathBuf) {
        self.histories.push(NavigationHistory::new(path));
        self.active = self.histories.len() - 1;
    }

    pub(in crate::desktop) fn close(&mut self, index: usize) -> bool {
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

    pub(in crate::desktop) fn rows(&self) -> Vec<TabRow> {
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
