use super::types::{ConversionError, ConversionKind};
#[cfg(windows)]
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn absolute_source(source: &Path) -> Result<PathBuf, ConversionError> {
    if source.is_absolute() {
        return Ok(source.to_path_buf());
    }
    std::env::current_dir()
        .map(|directory| directory.join(source))
        .map_err(|_| ConversionError::InvalidInput {
            path: source.to_path_buf(),
            reason: "não foi possível determinar o diretório atual",
        })
}

pub(crate) fn output_path(source: &Path, kind: ConversionKind) -> Result<PathBuf, ConversionError> {
    let source = absolute_source(source)?;
    let parent = source
        .parent()
        .ok_or_else(|| ConversionError::InvalidInput {
            path: source.clone(),
            reason: "o arquivo não possui diretório pai",
        })?;
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ConversionError::InvalidInput {
            path: source.clone(),
            reason: "o arquivo não possui nome compatível",
        })?;
    let extension = kind.extension();
    let mut destination = parent.join(format!("{stem}.{extension}"));
    if destination == source || same_existing_path(&source, &destination) {
        destination = parent.join(format!("{stem}.converted.{extension}"));
    }
    Ok(destination)
}

fn same_existing_path(source: &Path, destination: &Path) -> bool {
    #[cfg(windows)]
    {
        if !destination.exists() {
            return false;
        }
        match (fs::canonicalize(source), fs::canonicalize(destination)) {
            (Ok(source), Ok(destination)) => source == destination,
            _ => false,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (source, destination);
        false
    }
}

pub(crate) fn temporary_path(destination: &Path) -> Result<PathBuf, ConversionError> {
    let parent = destination
        .parent()
        .ok_or_else(|| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "a saída não possui diretório pai",
        })?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "a saída não possui nome válido",
        })?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "o relógio do sistema não pôde ser lido",
        })?
        .as_nanos();
    for attempt in 0..32_u32 {
        let candidate = parent.join(format!(
            ".{name}.rovex-convert-{}-{}",
            std::process::id(),
            timestamp + u128::from(attempt)
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ConversionError::InvalidInput {
        path: destination.to_path_buf(),
        reason: "não foi possível reservar um arquivo temporário",
    })
}
