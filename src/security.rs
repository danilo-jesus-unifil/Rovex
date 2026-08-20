#[cfg(windows)]
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DestinationPolicy {
    pub allow_existing_file: bool,
    pub allow_root: bool,
}

#[derive(Debug)]
pub enum ValidationError {
    EmptyPath,
    InvalidPath {
        path: PathBuf,
        reason: &'static str,
    },
    RootOperationDenied {
        path: PathBuf,
    },
    ExistingDestination {
        path: PathBuf,
    },
    ParentMissing {
        path: PathBuf,
    },
    SourceMissing {
        path: PathBuf,
    },
    UnsupportedSource {
        path: PathBuf,
    },
    SameSourceAndDestination {
        path: PathBuf,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(formatter, "o caminho não pode ser vazio"),
            Self::InvalidPath { path, reason } => {
                write!(
                    formatter,
                    "caminho inválido ({}): {}",
                    reason,
                    path.display()
                )
            }
            Self::RootOperationDenied { path } => {
                write!(
                    formatter,
                    "operação na raiz não permitida: {}",
                    path.display()
                )
            }
            Self::ExistingDestination { path } => {
                write!(formatter, "o destino já existe: {}", path.display())
            }
            Self::ParentMissing { path } => {
                write!(formatter, "o diretório pai não existe: {}", path.display())
            }
            Self::SourceMissing { path } => {
                write!(formatter, "a origem não existe: {}", path.display())
            }
            Self::UnsupportedSource { path } => {
                write!(
                    formatter,
                    "a origem não é um arquivo ou diretório suportado: {}",
                    path.display()
                )
            }
            Self::SameSourceAndDestination { path } => {
                write!(
                    formatter,
                    "origem e destino são o mesmo caminho: {}",
                    path.display()
                )
            }
            Self::Io {
                operation,
                path,
                kind,
            } => write!(
                formatter,
                "falha ao {} em {}: {:?}",
                operation,
                path.display(),
                kind
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

fn is_root_path(path: &Path) -> bool {
    path.parent().map(|parent| parent == path).unwrap_or(true)
}

fn destination_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(windows)]
fn is_reserved_windows_name(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    let trimmed = name.trim_end_matches([' ', '.']);
    let stem = trimmed.split('.').next().unwrap_or_default();
    let uppercase = stem.to_ascii_uppercase();
    matches!(
        uppercase.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "CLOCK$"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "COM¹"
            | "COM²"
            | "COM³"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
            | "LPT¹"
            | "LPT²"
            | "LPT³"
    )
}

pub fn validate_destination(
    source: Option<&Path>,
    destination: &Path,
    policy: DestinationPolicy,
) -> Result<PathBuf, ValidationError> {
    if destination.as_os_str().is_empty() {
        return Err(ValidationError::EmptyPath);
    }

    if destination.is_relative() {
        return Err(ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "caminho relativo ambíguo; use um caminho absoluto",
        });
    }

    if !policy.allow_root && is_root_path(destination) {
        return Err(ValidationError::RootOperationDenied {
            path: destination.to_path_buf(),
        });
    }

    let parent = destination_parent(destination);
    let parent_metadata = fs::symlink_metadata(parent).map_err(|error| ValidationError::Io {
        operation: "validar diretório pai",
        path: parent.to_path_buf(),
        kind: error.kind(),
    })?;
    if is_reparse_point(&parent_metadata) {
        return Err(ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "componente reparse point no diretório pai",
        });
    }
    if !parent_metadata.is_dir() {
        return Err(ValidationError::ParentMissing {
            path: parent.to_path_buf(),
        });
    }

    let mut current = PathBuf::new();
    let mut reached_root = false;
    for component in parent.components() {
        current.push(component.as_os_str());
        if matches!(component, Component::RootDir) {
            reached_root = true;
            continue;
        }
        if !reached_root {
            continue;
        }
        let metadata = fs::symlink_metadata(&current).map_err(|error| ValidationError::Io {
            operation: "validar componente do diretório pai",
            path: current.clone(),
            kind: error.kind(),
        })?;
        if is_reparse_point(&metadata) {
            return Err(ValidationError::InvalidPath {
                path: destination.to_path_buf(),
                reason: "componente reparse point no diretório pai",
            });
        }
    }

    let file_name = destination
        .file_name()
        .ok_or_else(|| ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "sem nome de arquivo",
        })?;
    if file_name == "." || file_name == ".." {
        return Err(ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "componente final ambíguo",
        });
    }
    #[cfg(windows)]
    if is_reserved_windows_name(file_name) {
        return Err(ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "nome reservado do Windows",
        });
    }

    let canonical_parent = fs::canonicalize(parent).map_err(|error| ValidationError::Io {
        operation: "normalizar diretório pai",
        path: parent.to_path_buf(),
        kind: error.kind(),
    })?;
    let normalized = canonical_parent.join(file_name);

    if let Some(source) = source {
        let canonical_source = fs::canonicalize(source).map_err(|error| ValidationError::Io {
            operation: "normalizar origem",
            path: source.to_path_buf(),
            kind: error.kind(),
        })?;
        if canonical_source == normalized {
            return Err(ValidationError::SameSourceAndDestination { path: normalized });
        }
    }

    if !policy.allow_existing_file && fs::symlink_metadata(&normalized).is_ok() {
        return Err(ValidationError::ExistingDestination { path: normalized });
    }

    Ok(normalized)
}

pub fn validate_source(path: &Path) -> Result<fs::FileType, ValidationError> {
    if path.as_os_str().is_empty() {
        return Err(ValidationError::EmptyPath);
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            ValidationError::SourceMissing {
                path: path.to_path_buf(),
            }
        } else {
            ValidationError::Io {
                operation: "validar origem",
                path: path.to_path_buf(),
                kind: error.kind(),
            }
        }
    })?;
    let file_type = metadata.file_type();
    if !(file_type.is_file() || file_type.is_dir() || file_type.is_symlink()) {
        return Err(ValidationError::UnsupportedSource {
            path: path.to_path_buf(),
        });
    }
    Ok(file_type)
}

pub fn ensure_not_root(path: &Path) -> Result<(), ValidationError> {
    if path.as_os_str().is_empty() || is_root_path(path) {
        return Err(ValidationError::RootOperationDenied {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
