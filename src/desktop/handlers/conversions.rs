use super::super::context::AppContext;
use super::dialogs;
use crate::converters::ConversionKind;
use std::sync::Arc;

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let pending_conversion = ctx.pending_conversion.clone();
    let directory_rows = Arc::clone(&ctx.directory_rows);
    let selection = Arc::clone(&ctx.selection);
    let tabs = ctx.tabs.clone();
    {
        let ui_weak = ui_weak.clone();
        let pending_conversion = pending_conversion.clone();
        let directory_rows = Arc::clone(&directory_rows);
        let selection = Arc::clone(&selection);
        let tabs = tabs.clone();
        ui.on_context_menu_convert_jxl_requested(move || {
            dialogs::show_conversion_dialog(
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
            dialogs::show_conversion_dialog(
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
            dialogs::show_conversion_dialog(
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
            dialogs::show_conversion_dialog(
                &ui_weak,
                &pending_conversion,
                &directory_rows,
                &selection,
                &tabs,
                ConversionKind::Flac,
            );
        });
    }
}
