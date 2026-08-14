use crate::security::{
    ensure_not_root, validate_destination, validate_source, DestinationPolicy, ValidationError,
};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyReport {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub bytes_copied: u64,
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
}

impl fmt::Display for OperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::FileSystem {
                operation,
                path,
                kind,
                raw_os_error,
            } => write!(
                formatter,
                "falha ao {} em {}: {:?} (código {:?})",
                operation,
                path.display(),
                kind,
                raw_os_error
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
        }
    }
}

impl std::error::Error for OperationError {}

impl From<ValidationError> for OperationError {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

fn from_io(operation: &'static str, path: &Path, error: io::Error) -> OperationError {
    OperationError::FileSystem {
        operation,
        path: path.to_path_buf(),
        kind: error.kind(),
        raw_os_error: error.raw_os_error(),
    }
}

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

pub fn copy_file_atomic(source: &Path, destination: &Path) -> Result<CopyReport, OperationError> {
    let source_type = validate_source(source)?;
    if !source_type.is_file() {
        return Err(OperationError::Validation(
            ValidationError::UnsupportedSource {
                path: source.to_path_buf(),
            },
        ));
    }

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
        let bytes_copied = io::copy(&mut input, &mut output)
            .map_err(|error| from_io("copiar arquivo", &temporary, error))?;
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

        fs::rename(&temporary, &destination)
            .map_err(|error| from_io("publicar arquivo", &destination, error))?;
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

pub fn create_directory(path: &Path) -> Result<(), OperationError> {
    let path = validate_destination(None, path, DestinationPolicy::default())?;
    fs::create_dir(&path).map_err(|error| from_io("criar diretório", &path, error))
}

pub fn rename_entry(source: &Path, destination: &Path) -> Result<(), OperationError> {
    validate_source(source)?;
    let destination =
        validate_destination(Some(source), destination, DestinationPolicy::default())?;
    fs::rename(source, destination).map_err(|error| from_io("renomear entrada", source, error))
}

fn is_directory_not_empty(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(39) | Some(145))
}

pub fn delete_entry(path: &Path) -> Result<(), OperationError> {
    ensure_not_root(path)?;
    let file_type = validate_source(path)?;
    if file_type.is_symlink() || file_type.is_file() {
        fs::remove_file(path).map_err(|error| from_io("excluir entrada", path, error))?;
        return Ok(());
    }

    if file_type.is_dir() {
        match fs::remove_dir(path) {
            Ok(()) => Ok(()),
            Err(error) if is_directory_not_empty(&error) => {
                Err(OperationError::DirectoryNotEmpty {
                    path: path.to_path_buf(),
                })
            }
            Err(error) => Err(from_io("excluir diretório", path, error)),
        }
    } else {
        Err(OperationError::Validation(
            ValidationError::UnsupportedSource {
                path: path.to_path_buf(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{copy_file_atomic, create_directory, delete_entry, rename_entry, OperationError};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("o relógio deve estar disponível")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rovex-operation-test-{unique}"));
        fs::create_dir(&path).expect("o diretório deve ser criado");
        path
    }

    #[test]
    fn copia_e_publica_arquivo_somente_apos_validacao() {
        let root = temporary_directory();
        let source = root.join("origem.txt");
        let destination = root.join("destino.txt");
        fs::write(&source, b"dados locais").expect("a origem deve ser criada");

        let report = copy_file_atomic(&source, &destination).expect("a cópia deve funcionar");
        assert_eq!(report.bytes_copied, 12);
        assert_eq!(
            fs::read(&destination).expect("o destino deve existir"),
            b"dados locais"
        );
        assert!(!root.join(".destino.txt.rovex-tmp").exists());
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }

    #[test]
    fn nao_sobrescreve_destino_existente() {
        let root = temporary_directory();
        let source = root.join("origem.txt");
        let destination = root.join("destino.txt");
        fs::write(&source, b"novo").expect("a origem deve ser criada");
        fs::write(&destination, b"antigo").expect("o destino deve ser criado");

        let result = copy_file_atomic(&source, &destination);
        assert!(matches!(result, Err(OperationError::Validation(_))));
        assert_eq!(
            fs::read(&destination).expect("o destino deve permanecer"),
            b"antigo"
        );
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }

    #[test]
    fn renomeia_cria_e_exclui_entrada() {
        let root = temporary_directory();
        let directory = root.join("nova-pasta");
        create_directory(&directory).expect("a pasta deve ser criada");
        let source = root.join("origem.txt");
        let destination = root.join("renomeado.txt");
        fs::write(&source, b"conteudo").expect("a origem deve ser criada");
        rename_entry(&source, &destination).expect("a entrada deve ser renomeada");
        delete_entry(&destination).expect("o arquivo deve ser excluído");
        delete_entry(&directory).expect("a pasta vazia deve ser excluída");
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }
}
