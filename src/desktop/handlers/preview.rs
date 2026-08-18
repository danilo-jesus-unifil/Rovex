use super::super::context::AppContext;
use super::super::jobs::{PreviewEvent, PreviewScheduler};
use super::super::state::{SharedRows, SharedSelection};
use crate::preview::PreviewLimits;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};
use std::path::PathBuf;
use std::sync::Arc;

fn selected_target(
    rows: &SharedRows,
    selection: &SharedSelection,
) -> Option<(PathBuf, String, bool)> {
    let selection = selection.lock().ok()?;
    let rows = rows.lock().ok()?;
    let mut selected = rows
        .iter()
        .filter(|row| selection.selected.contains(&row.key))
        .map(|row| (row.path.clone(), row.name.clone(), row.is_directory));
    let target = selected.next()?;
    if selected.next().is_some() {
        return None;
    }
    Some(target)
}

fn hide_preview(ui: &super::super::MainWindow) {
    ui.set_preview_visible(false);
    ui.set_preview_title(SharedString::default());
    ui.set_preview_status(SharedString::default());
    ui.set_preview_text(SharedString::default());
    ui.set_preview_image(Image::default());
}

pub(in crate::desktop) fn refresh_selection(
    ui_weak: &slint::Weak<super::super::MainWindow>,
    scheduler: Option<&Arc<PreviewScheduler>>,
    rows: &SharedRows,
    selection: &SharedSelection,
) {
    let Some(ui) = ui_weak.upgrade() else {
        return;
    };
    let Some((path, name, is_directory)) = selected_target(rows, selection) else {
        if let Some(scheduler) = scheduler {
            scheduler.cancel();
        }
        hide_preview(&ui);
        return;
    };
    let Some(scheduler) = scheduler else {
        ui.set_preview_visible(true);
        ui.set_preview_title(SharedString::from(name));
        ui.set_preview_status("Pré-visualização indisponível.".into());
        return;
    };
    ui.set_preview_title(SharedString::from(name));
    ui.set_preview_text(SharedString::default());
    ui.set_preview_image(Image::default());
    ui.set_preview_visible(true);
    if is_directory {
        ui.set_preview_status("Pastas não possuem prévia de imagem.".into());
        return;
    }
    let requested_generation = scheduler
        .request(path, PreviewLimits::default(), {
            let ui_weak = ui_weak.clone();
            let scheduler = Arc::clone(scheduler);
            move |event| {
                let scheduler_for_event = Arc::clone(&scheduler);
                let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                    let event_generation = match &event {
                        PreviewEvent::Ready { generation, .. }
                        | PreviewEvent::ReadyText { generation, .. }
                        | PreviewEvent::Failed { generation, .. } => *generation,
                    };
                    if scheduler_for_event.current_generation() != event_generation {
                        return;
                    }
                    match event {
                        PreviewEvent::Ready { preview, .. } => {
                            let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                                &preview.rgba,
                                preview.width,
                                preview.height,
                            );
                            ui.set_preview_image(Image::from_rgba8(buffer));
                            ui.set_preview_text(SharedString::default());
                            ui.set_preview_title(SharedString::from(
                                preview
                                    .source
                                    .file_name()
                                    .map(|value| value.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "Prévia".to_owned()),
                            ));
                            ui.set_preview_status(SharedString::from(format!(
                                "{} • {}×{}",
                                preview.format, preview.width, preview.height
                            )));
                            ui.set_preview_visible(true);
                        }
                        PreviewEvent::ReadyText { preview, .. } => {
                            ui.set_preview_image(Image::default());
                            ui.set_preview_text(SharedString::from(preview.text));
                            ui.set_preview_title(SharedString::from(
                                preview
                                    .source
                                    .file_name()
                                    .map(|value| value.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "Texto".to_owned()),
                            ));
                            ui.set_preview_status(SharedString::from(format!(
                                "{} • {} bytes{}",
                                preview.encoding,
                                preview.bytes_read,
                                if preview.truncated {
                                    " • truncado"
                                } else {
                                    ""
                                }
                            )));
                            ui.set_preview_visible(true);
                        }
                        PreviewEvent::Failed { message, .. } => {
                            ui.set_preview_image(Image::default());
                            ui.set_preview_text(SharedString::default());
                            ui.set_preview_status(SharedString::from(message));
                            ui.set_preview_visible(true);
                        }
                    }
                });
            }
        })
        .ok();
    if requested_generation.is_some() {
        ui.set_preview_status("Carregando prévia…".into());
    } else {
        ui.set_preview_status("Pré-visualização indisponível.".into());
    }
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let ui_weak = ctx.ui_weak.clone();
    let scheduler = ctx.preview_scheduler.clone();
    registration_ui.on_preview_dismissed(move || {
        if let Some(scheduler) = scheduler.as_ref() {
            scheduler.cancel();
        }
        if let Some(ui) = ui_weak.upgrade() {
            hide_preview(&ui);
        }
    });
}
