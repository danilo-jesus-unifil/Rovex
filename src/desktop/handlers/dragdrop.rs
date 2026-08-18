use super::super::context::AppContext;
use super::super::jobs::{OperationKind, OperationRequest, OperationScheduler};
use crate::security::validate_source;
use slint::language::DragAction;
use slint::winit_030::{EventResult, WinitWindowAccessor, winit};
use slint::{ComponentHandle, DataTransfer, SharedString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn validate_external_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("O drop não contém um caminho.".to_owned());
    }
    if !path.is_absolute() {
        return Err("Apenas caminhos absolutos podem ser recebidos.".to_owned());
    }
    let file_type = validate_source(path).map_err(|error| error.to_string())?;
    if !file_type.is_file() {
        return Err("O drop de pastas ainda não está disponível; solte um arquivo.".to_owned());
    }
    Ok(())
}

fn external_file_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value.trim());
    validate_external_path(&path)?;
    Ok(path)
}

fn transfer_path(data: &DataTransfer) -> Option<PathBuf> {
    data.plain_text()
        .ok()
        .and_then(|text| external_file_path(text.as_str()).ok())
}

fn show_drop_error(ui: &super::super::MainWindow, message: String) {
    ui.set_external_drop_active(false);
    ui.set_status_text(SharedString::from(message));
}

fn start_copy(
    ui_weak: &slint::Weak<super::super::MainWindow>,
    operation_scheduler: Option<&Arc<OperationScheduler>>,
    source: PathBuf,
) -> bool {
    let Some(ui) = ui_weak.upgrade() else {
        return false;
    };
    let destination = PathBuf::from(ui.get_current_path().to_string());
    if !destination.is_absolute() {
        show_drop_error(&ui, "A pasta atual não é um caminho absoluto.".to_owned());
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(&destination) else {
        show_drop_error(&ui, "A pasta atual não está disponível.".to_owned());
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        show_drop_error(&ui, "A pasta atual não é um diretório seguro.".to_owned());
        return false;
    }
    let request = OperationRequest {
        kind: OperationKind::Copy,
        sources: vec![source.clone()],
        destination_directory: Some(destination.clone()),
        rename_name: None,
        refresh_path: destination,
    };
    let Some(scheduler) = operation_scheduler else {
        show_drop_error(&ui, "O worker de operações está indisponível.".to_owned());
        return false;
    };
    if scheduler.start(request).is_err() {
        show_drop_error(
            &ui,
            "Já existe uma operação em andamento; aguarde o resultado.".to_owned(),
        );
        return false;
    }
    ui.set_external_drop_active(false);
    ui.set_operation_dialog_visible(true);
    ui.set_operation_dialog_title("Copiar arquivo solto".into());
    ui.set_operation_dialog_message(SharedString::from(format!(
        "Copiando {} para a pasta atual…",
        source.display()
    )));
    ui.set_operation_dialog_input(SharedString::default());
    ui.set_operation_busy(true);
    ui.set_operation_close_only(false);
    ui.set_operation_needs_input(false);
    ui.set_operation_progress(0);
    ui.set_operation_progress_text("Preparando…".into());
    true
}

pub(in crate::desktop) fn register(ctx: &AppContext) {
    let Some(registration_ui) = ctx.ui_weak.upgrade() else {
        return;
    };
    let operation_scheduler = ctx.operation_scheduler.clone();
    registration_ui.on_drop_can_drop(move |data| {
        if transfer_path(&data).is_some() {
            DragAction::Copy
        } else {
            DragAction::None
        }
    });

    let ui_weak_for_drop = ctx.ui_weak.clone();
    let scheduler_for_drop = operation_scheduler.clone();
    registration_ui.on_drop_received(move |data| {
        let Some(path) = transfer_path(&data) else {
            if let Some(ui) = ui_weak_for_drop.upgrade() {
                ui.set_status_text("Solte um arquivo absoluto e existente.".into());
            }
            return DragAction::None;
        };
        if start_copy(&ui_weak_for_drop, scheduler_for_drop.as_ref(), path) {
            DragAction::Copy
        } else {
            DragAction::None
        }
    });

    let ui_weak_for_events = ctx.ui_weak.clone();
    let scheduler_for_events = operation_scheduler;
    slint::Timer::single_shot(Default::default(), move || {
        let Some(ui) = ui_weak_for_events.upgrade() else {
            return;
        };
        ui.window().on_winit_window_event(move |_window, event| {
            let Some(ui) = ui_weak_for_events.upgrade() else {
                return EventResult::PreventDefault;
            };
            match event {
                winit::event::WindowEvent::HoveredFile(path) => {
                    if validate_external_path(path).is_ok() {
                        ui.set_external_drop_active(true);
                        ui.set_status_text("Solte o arquivo para copiar na pasta atual".into());
                    } else {
                        ui.set_external_drop_active(false);
                    }
                    EventResult::PreventDefault
                }
                winit::event::WindowEvent::HoveredFileCancelled => {
                    ui.set_external_drop_active(false);
                    EventResult::PreventDefault
                }
                winit::event::WindowEvent::DroppedFile(path) => {
                    ui.set_external_drop_active(false);
                    if validate_external_path(path).is_ok() {
                        let accepted = start_copy(
                            &ui_weak_for_events,
                            scheduler_for_events.as_ref(),
                            path.clone(),
                        );
                        if !accepted {
                            ui.set_status_text("O arquivo não pôde ser copiado.".into());
                        }
                    } else {
                        ui.set_status_text("Solte um arquivo absoluto e existente.".into());
                    }
                    EventResult::PreventDefault
                }
                _ => EventResult::Propagate,
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::external_file_path;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_relative_drop_path() {
        let error = external_file_path("arquivo.txt").expect_err("relative path must be rejected");
        assert!(error.contains("absolutos"));
    }

    #[test]
    fn accepts_existing_absolute_regular_file() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rovex-drop-{suffix}.txt"));
        fs::write(&path, b"drop").expect("create temporary source");
        assert_eq!(
            external_file_path(&path.to_string_lossy()),
            Ok(path.clone())
        );
        fs::remove_file(path).expect("remove temporary source");
    }

    #[test]
    fn rejects_missing_absolute_file() {
        let path = std::env::temp_dir().join("rovex-drop-missing.txt");
        let error = external_file_path(&path.to_string_lossy()).expect_err("missing source");
        assert!(error.contains("não existe") || error.contains("validar"));
    }
}
