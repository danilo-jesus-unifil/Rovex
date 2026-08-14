use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DestinationPolicy {
    pub allow_existing_file: bool,
    pub allow_root: bool,
}

#[derive(Debug)]
pub enum ValidationError {
    EmptyPath,
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

pub fn validate_destination(
    source: Option<&Path>,
    destination: &Path,
    policy: DestinationPolicy,
) -> Result<PathBuf, ValidationError> {
    if destination.as_os_str().is_empty() {
        return Err(ValidationError::EmptyPath);
    }

    let destination = destination.to_path_buf();
    if !policy.allow_root && destination.parent().is_none() {
        return Err(ValidationError::RootOperationDenied { path: destination });
    }

    if let Some(source) = source {
        if source == destination {
            return Err(ValidationError::SameSourceAndDestination { path: destination });
        }
    }

    let parent = destination
        .parent()
        .ok_or_else(|| ValidationError::RootOperationDenied {
            path: destination.clone(),
        })?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|error| ValidationError::Io {
        operation: "validar diretório pai",
        path: parent.to_path_buf(),
        kind: error.kind(),
    })?;
    if !parent_metadata.is_dir() {
        return Err(ValidationError::ParentMissing {
            path: parent.to_path_buf(),
        });
    }

    if !policy.allow_existing_file && fs::symlink_metadata(&destination).is_ok() {
        return Err(ValidationError::ExistingDestination { path: destination });
    }

    Ok(destination)
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
    if path.as_os_str().is_empty() || path.parent().is_none() {
        return Err(ValidationError::RootOperationDenied {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_not_root, validate_destination, DestinationPolicy, ValidationError};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("o relógio deve estar disponível")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rovex-security-test-{unique}"));
        fs::create_dir(&path).expect("o diretório deve ser criado");
        path
    }

    #[test]
    fn recusa_destino_existente_por_padrao() {
        let root = temporary_directory();
        let destination = root.join("existente.txt");
        fs::write(&destination, b"conteudo").expect("o arquivo deve ser criado");

        let result = validate_destination(None, &destination, DestinationPolicy::default());
        assert!(matches!(
            result,
            Err(ValidationError::ExistingDestination { .. })
        ));
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }

    #[test]
    fn recusa_raiz() {
        let result = ensure_not_root(std::path::Path::new("/"));
        assert!(matches!(
            result,
            Err(ValidationError::RootOperationDenied { .. })
        ));
    }
}
