use super::backend::{backend_candidates, is_backend_file, is_backend_retryable_error};
use super::paths::{absolute_source, output_path, temporary_path};
use super::process::{map_destination_error, spawn_ffmpeg, validate_output};
use super::types::{ConversionError, ConversionKind, ConversionReport, ConversionStage};
use crate::operations::{OperationError, publish_file_no_replace};
use crate::security::{DestinationPolicy, validate_destination, validate_source};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn convert_file<F>(
    source: &Path,
    kind: ConversionKind,
    cancel: &AtomicBool,
    mut stage: F,
) -> Result<ConversionReport, ConversionError>
where
    F: FnMut(ConversionStage),
{
    let source_type =
        validate_source(source).map_err(|error| ConversionError::Operation(error.into()))?;
    if !source_type.is_file() || source_type.is_symlink() {
        return Err(ConversionError::InvalidInput {
            path: source.to_path_buf(),
            reason: "a conversão exige um arquivo regular; links simbólicos não são seguidos",
        });
    }
    let source = absolute_source(source)?;
    if !kind.accepts(&source) {
        return Err(ConversionError::InvalidInput {
            path: source,
            reason: "a extensão não corresponde ao conversor escolhido",
        });
    }
    if cancel.load(Ordering::Acquire) {
        return Err(ConversionError::Cancelled);
    }
    stage(ConversionStage::Starting);
    let destination = output_path(&source, kind)?;
    let destination =
        validate_destination(Some(&source), &destination, DestinationPolicy::default()).map_err(
            |error| map_destination_error(OperationError::Validation(error), &destination),
        )?;
    let temporary = temporary_path(&destination)?;
    let ffmpeg_candidates = backend_candidates("ffmpeg", None);
    let ffmpeg_paths = ffmpeg_candidates
        .iter()
        .filter(|candidate| is_backend_file(candidate))
        .cloned()
        .collect::<Vec<_>>();
    if ffmpeg_paths.is_empty() {
        return Err(ConversionError::BackendUnavailable {
            executable: "ffmpeg",
            attempts: ffmpeg_candidates.len(),
        });
    }

    let result = (|| {
        let mut last_backend_error = None;
        for ffmpeg in ffmpeg_paths {
            if cancel.load(Ordering::Acquire) {
                return Err(ConversionError::Cancelled);
            }
            let ffprobe_candidates = backend_candidates("ffprobe", ffmpeg.parent());
            let ffprobe_paths = ffprobe_candidates
                .iter()
                .filter(|candidate| is_backend_file(candidate))
                .cloned()
                .collect::<Vec<_>>();
            if ffprobe_paths.is_empty() {
                last_backend_error = Some(ConversionError::BackendUnavailable {
                    executable: "ffprobe",
                    attempts: ffprobe_candidates.len(),
                });
                continue;
            }

            for ffprobe in ffprobe_paths {
                if cancel.load(Ordering::Acquire) {
                    return Err(ConversionError::Cancelled);
                }
                let attempt = (|| {
                    spawn_ffmpeg(&ffmpeg, &source, &temporary, kind, cancel, &mut stage)?;
                    stage(ConversionStage::Validating);
                    validate_output(&ffprobe, &temporary, kind, cancel)
                })();
                match attempt {
                    Ok(()) => {
                        if cancel.load(Ordering::Acquire) {
                            return Err(ConversionError::Cancelled);
                        }
                        stage(ConversionStage::Publishing);
                        publish_file_no_replace(&temporary, &destination)
                            .map_err(|error| map_destination_error(error, &destination))?;
                        return Ok(ConversionReport {
                            source: source.clone(),
                            destination: destination.clone(),
                            codec: kind.expected_codec(),
                        });
                    }
                    Err(error) if is_backend_retryable_error(&error) => {
                        last_backend_error = Some(error);
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Err(
            last_backend_error.unwrap_or(ConversionError::BackendUnavailable {
                executable: "ffmpeg",
                attempts: ffmpeg_candidates.len(),
            }),
        )
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
