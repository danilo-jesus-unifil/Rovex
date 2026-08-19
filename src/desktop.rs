use crate::settings::{Settings, SettingsStore};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

slint::include_modules!();

mod context;
mod handlers;
mod jobs;
mod locations;
mod state;

use context::AppContext;
use handlers::update_tab_visuals;
use jobs::start_load;
use state::SortSpec;

pub fn run() -> Result<(), slint::PlatformError> {
    let settings_store = SettingsStore::discover();
    let settings = settings_store
        .as_ref()
        .map(SettingsStore::load_or_default)
        .unwrap_or_default();
    let initial_path = initial_path(&settings);
    let ui = MainWindow::new()?;
    let context = AppContext::new(&ui, initial_path.clone(), settings_store);
    apply_settings(&context, &settings);
    handlers::register_all(&context);
    update_tab_visuals(&context.ui_weak, &context.tab_model, &context.tabs.borrow());
    start_load(
        &context.ui_weak,
        initial_path,
        context.load_scheduler.as_ref(),
    );
    context.persist_settings();
    let run_result = ui.run();
    context.persist_settings();
    run_result
}

fn initial_path(settings: &Settings) -> PathBuf {
    let cli_path = env::args_os().nth(1).map(PathBuf::from);
    cli_path
        .or_else(|| settings.last_path.clone().filter(|path| is_directory(path)))
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn is_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
}

fn apply_settings(context: &AppContext, settings: &Settings) {
    if let Ok(mut options) = context.listing_options.lock() {
        options.show_hidden = settings.show_hidden_files;
        options.show_system = settings.show_hidden_files;
    }
    if let Ok(mut sort_spec) = context.sort_spec.lock() {
        *sort_spec = SortSpec::from_persisted(settings.sort_column, settings.sort_ascending);
    }
    if let Some(ui) = context.ui_weak.upgrade() {
        ui.set_show_hidden_files(settings.show_hidden_files);
        ui.set_sort_column(settings.sort_column);
        ui.set_sort_ascending(settings.sort_ascending);
    }
}
