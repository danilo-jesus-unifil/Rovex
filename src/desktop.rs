use slint::{Model, ModelRc, SharedString, VecModel};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use crate::converters::ConversionKind;

slint::include_modules!();

mod jobs;
mod locations;
mod state;

use jobs::{
    ConversionRequest, ConversionScheduler, FilterScheduler, LoadScheduler, OperationKind,
    OperationRequest, OperationScheduler, start_load,
};
use locations::default_locations;
use state::{
    LoadedRow, SelectionState, SharedRows, SharedSelection, TabManager, parent_directory,
    selected_paths, selection_status, update_selection_visuals, validate_rename_name,
};

fn show_selected_operation_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<OperationRequest>>>,
    directory_rows: &SharedRows,
    selection: &SharedSelection,
    tabs: &Rc<std::cell::RefCell<TabManager>>,
    kind: OperationKind,
) {
    let sources = selected_paths(directory_rows, selection);
    if sources.is_empty() {
        return;
    }
    let (title, message, input, needs_input) = match kind {
        OperationKind::Copy => (
            "Copiar itens",
            format!(
                "Confirme a cópia de {} item(ns). Informe um diretório de destino absoluto. Destinos existentes não serão sobrescritos.",
                sources.len()
            ),
            String::new(),
            true,
        ),
        OperationKind::Move => (
            "Mover itens",
            format!(
                "Confirme a movimentação de {} item(ns). Informe um diretório de destino absoluto. Destinos existentes não serão sobrescritos.",
                sources.len()
            ),
            String::new(),
            true,
        ),
        OperationKind::Rename => {
            let Some(source) = sources.first() else {
                return;
            };
            let name = jobs::operation_label(source);
            (
                "Renomear item",
                "Informe um único nome novo. Separadores de caminho, ponto e ponto-ponto não são permitidos.".to_owned(),
                name,
                true,
            )
        }
        OperationKind::Delete => (
            "Excluir itens",
            format!(
                "Confirme a exclusão de {} item(ns). A operação não é recursiva: diretórios não vazios serão preservados.",
                sources.len()
            ),
            String::new(),
            false,
        ),
    };
    let request = OperationRequest {
        kind,
        sources,
        destination_directory: None,
        rename_name: if kind == OperationKind::Rename {
            Some(input.clone())
        } else {
            None
        },
        refresh_path: tabs.borrow().active().current.clone(),
    };
    show_operation_dialog(
        ui_weak,
        pending,
        request,
        title,
        &message,
        &input,
        needs_input,
    );
}

fn show_conversion_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<ConversionRequest>>>,
    directory_rows: &SharedRows,
    selection: &SharedSelection,
    tabs: &Rc<std::cell::RefCell<TabManager>>,
    kind: ConversionKind,
) {
    let sources = selected_paths(directory_rows, selection);
    if sources.is_empty() {
        return;
    }
    let request = ConversionRequest {
        kind,
        sources: sources.clone(),
        refresh_path: tabs.borrow().active().current.clone(),
    };
    *pending.borrow_mut() = Some(request);
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_context_menu_visible(false);
        ui.set_context_menu_can_jxl(false);
        ui.set_context_menu_can_opus(false);
        ui.set_context_menu_can_png(false);
        ui.set_context_menu_can_flac(false);
        ui.set_operation_dialog_title("Converter arquivos".into());
        ui.set_operation_dialog_message(SharedString::from(format!(
            "Confirme a conversão de {} item(ns) para {}. A saída será criada no mesmo diretório e nunca sobrescreverá um arquivo existente.",
            sources.len(),
            kind.label()
        )));
        ui.set_operation_dialog_input(SharedString::default());
        ui.set_operation_needs_input(false);
        ui.set_operation_close_only(false);
        ui.set_operation_busy(false);
        ui.set_operation_progress(0);
        ui.set_operation_progress_text(SharedString::default());
        ui.set_operation_dialog_visible(true);
    }
}

fn show_operation_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<OperationRequest>>>,
    request: OperationRequest,
    title: &str,
    message: &str,
    input: &str,
    needs_input: bool,
) {
    *pending.borrow_mut() = Some(request);
    if let Some(ui) = ui_weak.upgrade() {
        ui.set_operation_dialog_title(SharedString::from(title));
        ui.set_operation_dialog_message(SharedString::from(message));
        ui.set_operation_dialog_input(SharedString::from(input));
        ui.set_operation_needs_input(needs_input);
        ui.set_operation_close_only(false);
        ui.set_operation_busy(false);
        ui.set_operation_progress(0);
        ui.set_operation_progress_text(SharedString::default());
        ui.set_context_menu_visible(false);
        ui.set_context_menu_can_jxl(false);
        ui.set_context_menu_can_opus(false);
        ui.set_context_menu_can_png(false);
        ui.set_context_menu_can_flac(false);
        ui.set_operation_dialog_visible(true);
    }
}

fn update_tab_visuals(
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

pub fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let entries = Rc::new(VecModel::<FileRow>::default());
    ui.set_entries(ModelRc::from(entries.clone()));
    let locations = Rc::new(VecModel::<LocationRow>::default());
    ui.set_locations(ModelRc::from(locations.clone()));
    let tab_model = Rc::new(VecModel::<TabRow>::default());
    ui.set_tabs(ModelRc::from(tab_model.clone()));

    let initial_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    locations.set_vec(
        default_locations(&initial_path)
            .into_iter()
            .map(|location| LocationRow {
                label: SharedString::from(location.label),
                path: SharedString::from(location.path.to_string_lossy().to_string()),
            })
            .collect::<Vec<_>>(),
    );
    ui.set_current_path(SharedString::from(
        initial_path.to_string_lossy().to_string(),
    ));
    ui.set_status_text("Carregando…".into());

    let ui_weak = ui.as_weak();
    let tabs = Rc::new(std::cell::RefCell::new(TabManager::new(
        initial_path.clone(),
    )));
    update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
    let directory_rows: SharedRows = Arc::new(Mutex::new(Arc::from(Vec::<LoadedRow>::new())));
    let selection: SharedSelection = Arc::new(Mutex::new(SelectionState::default()));
    let filter_generation = Arc::new(AtomicU64::new(0));
    let load_scheduler = LoadScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&selection),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();
    let filter_scheduler = FilterScheduler::new(
        ui_weak.clone(),
        Arc::clone(&directory_rows),
        Arc::clone(&selection),
        Arc::clone(&filter_generation),
    )
    .map(Arc::new)
    .ok();
    let pending_operation = Rc::new(std::cell::RefCell::new(None::<OperationRequest>));
    let pending_conversion = Rc::new(std::cell::RefCell::new(None::<ConversionRequest>));
    let operation_scheduler = OperationScheduler::new(ui_weak.clone(), load_scheduler.clone())
        .map(Arc::new)
        .ok();
    let conversion_scheduler = ConversionScheduler::new(ui_weak.clone(), load_scheduler.clone())
        .map(Arc::new)
        .ok();

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

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        ui.on_select_row(move |index, control, shift| {
            if index < 0 {
                return;
            }
            let Some(row) = entries.row_data(index as usize) else {
                return;
            };
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|visible_row| visible_row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.click(row.key.as_str(), &keys, control, shift);
            if let Some(ui) = ui_weak.upgrade() {
                if !update_selection_visuals(&ui, &state) {
                    ui.set_status_text("Falha interna ao atualizar a seleção".into());
                } else {
                    ui.set_selection_count(state.count() as i32);
                    ui.set_status_text(SharedString::from(selection_status(&state)));
                }
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        ui.on_select_all(move || {
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|row| row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.select_all(keys);
            if let Some(ui) = ui_weak.upgrade() {
                if !update_selection_visuals(&ui, &state) {
                    ui.set_status_text("Falha interna ao atualizar a seleção".into());
                } else {
                    ui.set_selection_count(state.count() as i32);
                    ui.set_status_text(SharedString::from(selection_status(&state)));
                }
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let entries = entries.clone();
        let selection = Arc::clone(&selection);
        ui.on_context_menu_requested(move |index| {
            if index < 0 {
                return;
            }
            let Some(row) = entries.row_data(index as usize) else {
                return;
            };
            let keys = (0..entries.row_count())
                .filter_map(|row_index| entries.row_data(row_index))
                .map(|visible_row| visible_row.key.to_string())
                .collect::<Vec<_>>();
            let Ok(mut state) = selection.lock() else {
                return;
            };
            state.click(row.key.as_str(), &keys, false, false);
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            if !update_selection_visuals(&ui, &state) {
                ui.set_status_text("Falha interna ao atualizar a seleção".into());
                return;
            }
            ui.set_selection_count(state.count() as i32);
            ui.set_status_text(SharedString::from(selection_status(&state)));
            let is_regular_file = row.kind == "Arquivo";
            ui.set_context_menu_target_name(row.name.clone());
            ui.set_context_menu_can_jxl(
                is_regular_file && ConversionKind::JpegXl.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_opus(
                is_regular_file && ConversionKind::Opus.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_png(
                is_regular_file && ConversionKind::Png.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_can_flac(
                is_regular_file && ConversionKind::Flac.accepts(Path::new(row.name.as_str())),
            );
            ui.set_context_menu_visible(true);
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_copy_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_move_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_rename_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_delete_requested(move || {
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Delete,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_copy_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Copy,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_move_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Move,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_rename_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Rename,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_delete_requested(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
            show_selected_operation_dialog(
                &ui_weak,
                &pending_operation,
                &directory_rows,
                &selection,
                &tabs,
                OperationKind::Delete,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_convert_jxl_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &tabs,
                ConversionKind::JpegXl,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_convert_opus_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &tabs,
                ConversionKind::Opus,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_convert_png_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &tabs,
                ConversionKind::Png,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_convert_flac_requested(move || {
            show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &tabs,
                ConversionKind::Flac,
            );
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let pending_conversion = pending_conversion.clone();
        let operation_scheduler = operation_scheduler.clone();
        let conversion_scheduler = conversion_scheduler.clone();
        ui.on_operation_confirmed(move || {
            let conversion_request = pending_conversion.borrow_mut().take();
            if let Some(request) = conversion_request {
                let Some(ui) = ui_weak.upgrade() else {
                    return;
                };
                let Some(scheduler) = conversion_scheduler.as_ref() else {
                    ui.set_operation_dialog_message(
                        "O worker de conversão está indisponível.".into(),
                    );
                    *pending_conversion.borrow_mut() = Some(request);
                    return;
                };
                if let Err(request) = scheduler.start(request) {
                    ui.set_operation_dialog_message(
                        "Já existe uma conversão em andamento; aguarde o resultado.".into(),
                    );
                    *pending_conversion.borrow_mut() = Some(request);
                    return;
                }
                ui.set_operation_busy(true);
                ui.set_operation_close_only(false);
                ui.set_operation_needs_input(false);
                ui.set_operation_progress(0);
                ui.set_operation_progress_text("Preparando conversão…".into());
                return;
            }

            let Some(mut request) = pending_operation.borrow_mut().take() else {
                return;
            };
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            match request.kind {
                OperationKind::Copy | OperationKind::Move => {
                    let input = ui.get_operation_dialog_input().to_string();
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        ui.set_operation_dialog_message("Informe um diretório de destino.".into());
                        *pending_operation.borrow_mut() = Some(request);
                        return;
                    }
                    request.destination_directory = Some(PathBuf::from(trimmed));
                }
                OperationKind::Rename => {
                    let input = ui.get_operation_dialog_input().to_string();
                    match validate_rename_name(&input) {
                        Ok(name) => request.rename_name = Some(name),
                        Err(error) => {
                            ui.set_operation_dialog_message(SharedString::from(error));
                            *pending_operation.borrow_mut() = Some(request);
                            return;
                        }
                    }
                }
                OperationKind::Delete => {}
            }
            let Some(scheduler) = operation_scheduler.as_ref() else {
                ui.set_operation_dialog_message("O worker de operações está indisponível.".into());
                *pending_operation.borrow_mut() = Some(request);
                return;
            };
            if let Err(request) = scheduler.start(request) {
                ui.set_operation_dialog_message(
                    "Já existe uma operação em andamento; aguarde o resultado.".into(),
                );
                *pending_operation.borrow_mut() = Some(request);
                return;
            }
            ui.set_operation_busy(true);
            ui.set_operation_close_only(false);
            ui.set_operation_needs_input(false);
            ui.set_operation_progress(0);
            ui.set_operation_progress_text("Preparando…".into());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let operation_scheduler = operation_scheduler.clone();
        let conversion_scheduler = conversion_scheduler.clone();
        ui.on_operation_cancelled(move || {
            if let Some(scheduler) = operation_scheduler.as_ref() {
                scheduler.cancel();
            }
            if let Some(scheduler) = conversion_scheduler.as_ref() {
                scheduler.cancel();
            }
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_operation_progress_text("Cancelamento solicitado…".into());
                ui.set_operation_dialog_message(
                    "A tarefa será interrompida no próximo ponto seguro; o resultado parcial será verificado.".into(),
                );
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let pending_operation = pending_operation.clone();
        let pending_conversion = pending_conversion.clone();
        ui.on_operation_dismissed(move || {
            if let Some(ui) = ui_weak.upgrade()
                && !ui.get_operation_busy()
            {
                ui.set_operation_dialog_visible(false);
                *pending_operation.borrow_mut() = None;
                *pending_conversion.borrow_mut() = None;
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        ui.on_context_menu_dismissed(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_context_menu_visible(false);
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let filter_generation = Arc::clone(&filter_generation);
        let filter_scheduler = filter_scheduler.clone();
        let selection = Arc::clone(&selection);
        ui.on_filter_changed(move |text| {
            if let Ok(mut state) = selection.lock() {
                state.clear();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_selection_count(0);
                    ui.set_focused_row_index(-1);
                }
            }
            let generation = filter_generation.fetch_add(1, Ordering::AcqRel) + 1;
            let query = text.to_string();
            let Some(scheduler) = filter_scheduler.as_ref() else {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Filtro indisponível".into());
                });
                return;
            };
            if scheduler.schedule(generation, query).is_err() {
                let _ = ui_weak.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("Falha ao agendar o filtro".into());
                });
            }
        });
    }

    update_tab_visuals(&ui_weak, &tab_model, &tabs.borrow());
    start_load(&ui_weak, initial_path, load_scheduler.as_ref());
    ui.run()
}
