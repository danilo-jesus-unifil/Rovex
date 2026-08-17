use super::super::context::AppContext;

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let operation_scheduler = ctx.operation_scheduler.clone();
    let conversion_scheduler = ctx.conversion_scheduler.clone();
    let pending_operation = ctx.pending_operation.clone();
    let pending_conversion = ctx.pending_conversion.clone();
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
}
