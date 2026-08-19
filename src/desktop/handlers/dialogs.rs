use super::super::MainWindow;
use super::super::jobs;
use super::super::jobs::{ConversionRequest, OperationKind, OperationRequest};
use super::super::state::{SharedRows, SharedSelection, TabManager, selected_paths};
use crate::converters::ConversionKind;
use slint::SharedString;
use std::rc::Rc;

pub(in crate::desktop) fn show_selected_operation_dialog(
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
                "Confirme a exclusão de {} item(ns). No Windows, os itens serão enviados à Lixeira; diretórios não vazios serão preservados.",
                sources.len()
            ),
            String::new(),
            false,
        ),
        OperationKind::CreateDirectory => return,
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

pub(in crate::desktop) fn show_create_directory_dialog(
    ui_weak: &slint::Weak<MainWindow>,
    pending: &Rc<std::cell::RefCell<Option<OperationRequest>>>,
    tabs: &Rc<std::cell::RefCell<TabManager>>,
) {
    let current = tabs.borrow().active().current.clone();
    let request = OperationRequest {
        kind: OperationKind::CreateDirectory,
        sources: Vec::new(),
        destination_directory: None,
        rename_name: None,
        refresh_path: current,
    };
    show_operation_dialog(
        ui_weak,
        pending,
        request,
        "Nova pasta",
        "Informe um único nome para a nova pasta. Separadores de caminho, ponto e ponto-ponto não são permitidos.",
        "",
        true,
    );
}

pub(in crate::desktop) fn show_conversion_dialog(
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
        ui.set_context_menu_can_terminal(false);
        ui.set_context_menu_can_open_with(false);
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

pub(in crate::desktop) fn show_operation_dialog(
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
        ui.set_context_menu_can_terminal(false);
        ui.set_context_menu_can_open_with(false);
        ui.set_operation_dialog_visible(true);
    }
}
