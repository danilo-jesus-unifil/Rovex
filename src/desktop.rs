use std::path::PathBuf;

slint::include_modules!();

mod context;
mod handlers;
mod jobs;
mod locations;
mod state;

use context::AppContext;
use handlers::update_tab_visuals;
use jobs::start_load;

pub fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let initial_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let context = AppContext::new(&ui, initial_path.clone());
    handlers::register_all(&context);
    update_tab_visuals(&context.ui_weak, &context.tab_model, &context.tabs.borrow());
    start_load(
        &context.ui_weak,
        initial_path,
        context.load_scheduler.as_ref(),
    );
    ui.run()
}
