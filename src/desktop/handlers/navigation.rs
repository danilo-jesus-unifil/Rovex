use super::super::context::AppContext;
use super::super::jobs::start_load;
use super::super::state::TabManager;
use super::super::state::parent_directory;
use super::super::{MainWindow, TabRow};
use slint::{Model, SharedString, VecModel};
use std::path::PathBuf;
use std::sync::Arc;

pub(in crate::desktop) fn update_tab_visuals(
    ui_weak: &slint::Weak<MainWindow>,
    tab_model: &VecModel<TabRow>,
    tabs: &TabManager,
) {
    tab_model.set_vec(tabs.rows());
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_current_path(SharedString::from(
            tabs.active().current.to_string_lossy().to_string(),
        ));
        ui.set_can_go_back(tabs.active().can_go_back());
        ui.set_can_go_forward(tabs.active().can_go_forward());
    }
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let tab_model = ctx.tab_model.clone();
    let tabs = ctx.tabs.clone();
    let load_scheduler = ctx.load_scheduler.clone();
    let locations = ctx.locations.clone();
    let entries = ctx.entries.clone();
    let directory_rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);
    {
        let ui_weak = ui_weak.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_refresh_requested(move || {
            let path = tabs.borrow().active().current.clone();
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_to(move |text| {
            let path = PathBuf::from(text.to_string());
            let changed = tabs.borrow_mut().active_mut().visit(path.clone());
            if changed {
                update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            }
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let locations = locations.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_to_location(move |index| {
            if index < 0 {
                return;
            }
            let Some(location) = locations.row_data(index as usize) else {
                return;
            };
            let path = PathBuf::from(location.path.to_string());
            tabs.borrow_mut().active_mut().visit(path.clone());
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_back_requested(move || {
            let Some(path) = tabs.borrow_mut().active_mut().go_back() else {
                return;
            };
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_forward_requested(move || {
            let Some(path) = tabs.borrow_mut().active_mut().go_forward() else {
                return;
            };
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        ui.on_navigate_up(move || {
            let current = tabs.borrow().active().current.clone();
            let Some(parent) = parent_directory(&current) else {
                return;
            };
            tabs.borrow_mut().active_mut().visit(parent.clone());
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, parent, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let entries = entries.clone();
        let tabs = tabs.clone();
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
            tabs.borrow_mut().active_mut().visit(next.clone());
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, next, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        let selection = Arc::clone(&selection);
        ui.on_new_tab_requested(move || {
            let path = tabs.borrow().active().current.clone();
            tabs.borrow_mut().new_tab(path.clone());
            if let Ok(mut state) = selection.lock() {
                state.clear();
            }
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        let selection = Arc::clone(&selection);
        ui.on_select_tab(move |index| {
            if index < 0 || !tabs.borrow_mut().select(index as usize) {
                return;
            }
            let path = tabs.borrow().active().current.clone();
            if let Ok(mut state) = selection.lock() {
                state.clear();
            }
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let tab_model = tab_model.clone();
        let tabs = tabs.clone();
        let load_scheduler = load_scheduler.clone();
        let selection = Arc::clone(&selection);
        ui.on_close_tab(move |index| {
            if index < 0 || !tabs.borrow_mut().close(index as usize) {
                return;
            }
            let path = tabs.borrow().active().current.clone();
            if let Ok(mut state) = selection.lock() {
                state.clear();
            }
            update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
            start_load(&ui_weak, path, load_scheduler.as_ref());
        });
    }
}
