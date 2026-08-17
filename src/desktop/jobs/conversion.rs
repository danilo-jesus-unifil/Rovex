use super::operations::{emit_item_progress, emit_stage_progress, operation_label};
use super::types::{ConversionOutcome, ConversionRequest, OperationUpdate};
use crate::converters::{ConversionError, convert_file};
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::desktop) fn execute_conversion<F>(
    request: &ConversionRequest,
    cancel: &AtomicBool,
    mut emit: F,
) -> ConversionOutcome
where
    F: FnMut(OperationUpdate),
{
    let total_items = request.sources.len();
    let mut completed = 0;
    let mut failed = Vec::new();

    for (index, source) in request.sources.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            return ConversionOutcome {
                kind: request.kind,
                completed,
                failed,
                cancelled: true,
            };
        }
        let label = operation_label(source);
        let result = convert_file(source, request.kind, cancel, |stage| {
            emit_stage_progress(&mut emit, index, total_items, &label, stage);
        });
        match result {
            Ok(_) => {
                completed += 1;
                emit_item_progress(&mut emit, index + 1, total_items, &label, 0, 0);
            }
            Err(ConversionError::Cancelled) => {
                return ConversionOutcome {
                    kind: request.kind,
                    completed,
                    failed,
                    cancelled: true,
                };
            }
            Err(error) => failed.push(format!("{label}: {error}")),
        }
    }

    ConversionOutcome {
        kind: request.kind,
        completed,
        failed,
        cancelled: false,
    }
}
