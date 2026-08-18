use super::super::context::AppContext;
use super::super::jobs::OperationKind;
use super::super::state::validate_rename_name;
use slint::SharedString;
use std::path::PathBuf;
pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let pending_operation = ctx.pending_operation.clone();
    let pending_conversion = ctx.pending_conversion.clone();
    let operation_scheduler = ctx.operation_scheduler.clone();
    let conversion_scheduler = ctx.conversion_scheduler.clone();
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
                OperationKind::CreateDirectory => {
                    let input = ui.get_operation_dialog_input().to_string();
                    match validate_rename_name(&input) {
                        Ok(name) => request.sources = vec![request.refresh_path.join(name)],
                        Err(error) => {
                            ui.set_operation_dialog_message(SharedString::from(error));
                            *pending_operation.borrow_mut() = Some(request);
                            return;
                        }
                    }
                }
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
}
