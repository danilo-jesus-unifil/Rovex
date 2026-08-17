use crate::security::ValidationError;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyReport {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub bytes_copied: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyProgress {
    pub bytes_copied: u64,
    pub total_bytes: u64,
}

#[derive(Debug)]
pub enum OperationError {
    Validation(ValidationError),
    FileSystem {
        operation: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
        raw_os_error: Option<i32>,
    },
    DirectoryNotEmpty {
        path: PathBuf,
    },
    OutputValidationFailed {
        path: PathBuf,
    },
    Cancelled,
}

fn human_io_reason(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::NotFound => "o caminho não foi encontrado",
        io::ErrorKind::PermissionDenied => "o acesso foi negado",
        io::ErrorKind::AlreadyExists => "o destino já existe",
        io::ErrorKind::InvalidInput | io::ErrorKind::InvalidFilename => "o caminho não é válido",
        io::ErrorKind::DirectoryNotEmpty => "o diretório não está vazio",
        _ => "ocorreu um erro de sistema",
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::FileSystem {
                operation,
                path,
                kind,
                ..
            } => write!(
                formatter,
                "não foi possível {} em {}: {}",
                operation,
                path.display(),
                human_io_reason(*kind)
            ),
            Self::DirectoryNotEmpty { path } => {
                write!(formatter, "o diretório não está vazio: {}", path.display())
            }
            Self::OutputValidationFailed { path } => {
                write!(
                    formatter,
                    "o arquivo gerado não passou pela validação: {}",
                    path.display()
                )
            }
            Self::Cancelled => write!(formatter, "operação cancelada pelo usuário"),
        }
    }
}

impl OperationError {
    pub fn is_cross_device(&self) -> bool {
        matches!(
            self,
            Self::FileSystem {
                kind: io::ErrorKind::CrossesDevices,
                ..
            } | Self::FileSystem {
                raw_os_error: Some(17 | 18),
                ..
            }
        )
    }
}

impl std::error::Error for OperationError {}

impl From<ValidationError> for OperationError {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

pub(super) fn from_io(operation: &'static str, path: &Path, error: io::Error) -> OperationError {
    OperationError::FileSystem {
        operation,
        path: path.to_path_buf(),
        kind: error.kind(),
        raw_os_error: error.raw_os_error(),
    }
}
