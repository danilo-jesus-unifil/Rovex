use super::error::{CopyProgress, CopyReport, OperationError, from_io};
use crate::security::{DestinationPolicy, ValidationError, validate_destination, validate_source};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_destination(destination: &Path) -> Result<PathBuf, OperationError> {
    let parent = destination.parent().ok_or_else(|| {
        OperationError::Validation(ValidationError::ParentMissing {
            path: destination.to_path_buf(),
        })
    })?;
    let file_name = destination
        .file_name()
        .ok_or(OperationError::Validation(ValidationError::EmptyPath))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OperationError::OutputValidationFailed {
            path: destination.to_path_buf(),
        })?
        .as_nanos();

    for attempt in 0..32_u32 {
        let candidate = parent.join(format!(
            ".{}.rovex-tmp-{}-{}",
            file_name.to_string_lossy(),
            std::process::id(),
            timestamp + u128::from(attempt)
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(OperationError::FileSystem {
        operation: "reservar arquivo temporário",
        path: destination.to_path_buf(),
        kind: io::ErrorKind::AlreadyExists,
        raw_os_error: None,
    })
}

pub(crate) fn copy_temporary_no_replace(
    temporary: &Path,
    destination: &Path,
) -> Result<(), OperationError> {
    let mut input =
        File::open(temporary).map_err(|error| from_io("abrir temporário", temporary, error))?;
    let mut output = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
    {
        Ok(output) => output,
        Err(error) => {
            return Err(from_io("criar destino sem sobrescrita", destination, error));
        }
    };
    let result = (|| {
        let bytes_copied = io::copy(&mut input, &mut output)
            .map_err(|error| from_io("publicar conteúdo", destination, error))?;
        output
            .flush()
            .and_then(|_| output.sync_all())
            .map_err(|error| from_io("sincronizar destino", destination, error))?;
        drop(output);

        let metadata = fs::metadata(temporary)
            .map_err(|error| from_io("validar temporário", temporary, error))?;
        if metadata.len() != bytes_copied {
            return Err(OperationError::OutputValidationFailed {
                path: destination.to_path_buf(),
            });
        }
        fs::remove_file(temporary)
            .map_err(|error| from_io("limpar arquivo temporário", temporary, error))?;
        Ok(())
    })();
    if result.is_err() {
        // `create_new` succeeded before entering the closure, so this file is
        // owned by this operation and may be cleaned up safely.
        let _ = fs::remove_file(destination);
    }
    result
}

pub(crate) fn publish_file_no_replace(
    temporary: &Path,
    destination: &Path,
) -> Result<(), OperationError> {
    match fs::hard_link(temporary, destination) {
        Ok(()) => fs::remove_file(temporary)
            .map_err(|error| from_io("limpar arquivo temporário", temporary, error)),
        Err(error)
            if error.kind() == io::ErrorKind::AlreadyExists
                || fs::symlink_metadata(destination).is_ok() =>
        {
            Err(from_io(
                "publicar arquivo sem sobrescrita",
                destination,
                error,
            ))
        }
        Err(_) => copy_temporary_no_replace(temporary, destination),
    }
}

pub fn copy_file_atomic(source: &Path, destination: &Path) -> Result<CopyReport, OperationError> {
    let cancel = AtomicBool::new(false);
    copy_file_atomic_with_progress(source, destination, &cancel, |_| {})
}

pub fn copy_file_atomic_with_progress<F>(
    source: &Path,
    destination: &Path,
    cancel: &AtomicBool,
    mut progress: F,
) -> Result<CopyReport, OperationError>
where
    F: FnMut(CopyProgress),
{
    let source_type = validate_source(source)?;
    if !source_type.is_file() {
        return Err(OperationError::Validation(
            ValidationError::UnsupportedSource {
                path: source.to_path_buf(),
            },
        ));
    }
    if cancel.load(Ordering::Acquire) {
        return Err(OperationError::Cancelled);
    }

    let total_bytes = fs::metadata(source)
        .map_err(|error| from_io("ler tamanho da origem", source, error))?
        .len();
    let destination =
        validate_destination(Some(source), destination, DestinationPolicy::default())?;
    let temporary = temporary_destination(&destination)?;
    let result = (|| {
        let mut input =
            File::open(source).map_err(|error| from_io("abrir origem", source, error))?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| from_io("criar arquivo temporário", &temporary, error))?;
        let mut buffer = [0_u8; 64 * 1024];
        let mut bytes_copied = 0_u64;
        loop {
            if cancel.load(Ordering::Acquire) {
                return Err(OperationError::Cancelled);
            }
            let read = input
                .read(&mut buffer)
                .map_err(|error| from_io("ler origem", source, error))?;
            if read == 0 {
                break;
            }
            output
                .write_all(&buffer[..read])
                .map_err(|error| from_io("copiar arquivo", &temporary, error))?;
            bytes_copied += read as u64;
            progress(CopyProgress {
                bytes_copied,
                total_bytes,
            });
        }
        output
            .flush()
            .and_then(|_| output.sync_all())
            .map_err(|error| from_io("sincronizar arquivo temporário", &temporary, error))?;
        drop(output);

        let metadata = fs::metadata(&temporary)
            .map_err(|error| from_io("validar arquivo temporário", &temporary, error))?;
        if metadata.len() != bytes_copied {
            return Err(OperationError::OutputValidationFailed {
                path: temporary.clone(),
            });
        }
        if cancel.load(Ordering::Acquire) {
            return Err(OperationError::Cancelled);
        }

        publish_file_no_replace(&temporary, &destination)?;
        Ok(CopyReport {
            source: source.to_path_buf(),
            destination: destination.clone(),
            bytes_copied,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
