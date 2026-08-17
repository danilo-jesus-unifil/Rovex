use super::types::{OperationKind, OperationOutcome, OperationRequest, OperationUpdate};
use crate::converters::ConversionStage;
use crate::operations::{
    CopyProgress, OperationError, copy_file_atomic_with_progress, delete_entry, rename_entry,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::desktop) fn operation_label(source: &Path) -> String {
    source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.display().to_string())
}

fn operation_destination(
    request: &OperationRequest,
    source: &Path,
) -> Result<PathBuf, OperationError> {
    let directory = request
        .destination_directory
        .as_ref()
        .ok_or(OperationError::Validation(
            crate::security::ValidationError::EmptyPath,
        ))?;
    let file_name = source.file_name().ok_or(OperationError::Validation(
        crate::security::ValidationError::EmptyPath,
    ))?;
    Ok(directory.join(file_name))
}

pub(in crate::desktop) fn emit_item_progress<F>(
    emit: &mut F,
    index: usize,
    total_items: usize,
    label: &str,
    current_bytes: u64,
    current_total_bytes: u64,
) where
    F: FnMut(OperationUpdate),
{
    emit(OperationUpdate {
        completed_items: index,
        total_items,
        current_bytes,
        current_total_bytes,
        explicit_percent: None,
        label: label.to_owned(),
    });
}

pub(in crate::desktop) fn emit_stage_progress<F>(
    emit: &mut F,
    index: usize,
    total_items: usize,
    label: &str,
    stage: ConversionStage,
) where
    F: FnMut(OperationUpdate),
{
    emit(OperationUpdate {
        completed_items: index,
        total_items,
        current_bytes: 0,
        current_total_bytes: 0,
        explicit_percent: Some(stage.percent()),
        label: label.to_owned(),
    });
}

pub(in crate::desktop) fn execute_operation<F>(
    request: &OperationRequest,
    cancel: &AtomicBool,
    mut emit: F,
) -> OperationOutcome
where
    F: FnMut(OperationUpdate),
{
    let total_items = request.sources.len();
    let mut completed = 0;
    let mut failed = Vec::new();

    for (index, source) in request.sources.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            return OperationOutcome {
                kind: request.kind,
                completed,
                failed,
                cancelled: true,
            };
        }
        let label = operation_label(source);
        let result = match request.kind {
            OperationKind::Copy => {
                let destination = operation_destination(request, source);
                match destination {
                    Ok(destination) => copy_file_atomic_with_progress(
                        source,
                        &destination,
                        cancel,
                        |CopyProgress {
                             bytes_copied,
                             total_bytes,
                         }| {
                            emit_item_progress(
                                &mut emit,
                                index,
                                total_items,
                                &label,
                                bytes_copied,
                                total_bytes,
                            );
                        },
                    )
                    .map(|_| ()),
                    Err(error) => Err(error),
                }
            }
            OperationKind::Move => {
                let destination = operation_destination(request, source);
                match destination {
                    Ok(destination) => match rename_entry(source, &destination) {
                        Ok(()) => Ok(()),
                        Err(error) if error.is_cross_device() => {
                            let copy_result = copy_file_atomic_with_progress(
                                source,
                                &destination,
                                cancel,
                                |CopyProgress {
                                     bytes_copied,
                                     total_bytes,
                                 }| {
                                    emit_item_progress(
                                        &mut emit,
                                        index,
                                        total_items,
                                        &label,
                                        bytes_copied,
                                        total_bytes,
                                    );
                                },
                            );
                            match copy_result {
                                Ok(_) if cancel.load(Ordering::Acquire) => {
                                    return OperationOutcome {
                                        kind: request.kind,
                                        completed,
                                        failed: vec![format!(
                                            "{label}: cópia concluída, origem preservada porque o cancelamento foi solicitado"
                                        )],
                                        cancelled: true,
                                    };
                                }
                                Ok(_) => delete_entry(source),
                                Err(error) => Err(error),
                            }
                        }
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                }
            }
            OperationKind::Rename => {
                let Some(name) = request.rename_name.as_deref() else {
                    return OperationOutcome {
                        kind: request.kind,
                        completed,
                        failed: vec![format!("{label}: novo nome ausente")],
                        cancelled: false,
                    };
                };
                let Some(parent) = source.parent() else {
                    return OperationOutcome {
                        kind: request.kind,
                        completed,
                        failed: vec![format!("{label}: diretório pai ausente")],
                        cancelled: false,
                    };
                };
                rename_entry(source, &parent.join(name))
            }
            OperationKind::Delete => delete_entry(source),
        };

        match result {
            Ok(()) => {
                completed += 1;
                emit_item_progress(&mut emit, index + 1, total_items, &label, 0, 0);
            }
            Err(OperationError::Cancelled) => {
                return OperationOutcome {
                    kind: request.kind,
                    completed,
                    failed,
                    cancelled: true,
                };
            }
            Err(error) => failed.push(format!("{label}: {error}")),
        }
    }

    OperationOutcome {
        kind: request.kind,
        completed,
        failed,
        cancelled: false,
    }
}
