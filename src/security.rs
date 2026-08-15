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
    if parent_metadata.file_type().is_symlink() {
        return Err(ValidationError::InvalidPath {
            path: destination.to_path_buf(),
            reason: "componente symlink no diretório pai",
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
        if metadata.file_type().is_symlink() {
            return Err(ValidationError::InvalidPath {
                path: destination.to_path_buf(),
                reason: "componente symlink no diretório pai",
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
mod tests {
    use super::{DestinationPolicy, ValidationError, ensure_not_root, validate_destination};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory() -> std::path::PathBuf {
        let unique = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rovex-security-test-{}-{unique}",
            std::process::id()
        ));
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

    #[test]
    fn normaliza_parent_no_diretorio_pai() {
        let root = temporary_directory();
        let nested = root.join("nested");
        fs::create_dir(&nested).expect("a pasta aninhada deve ser criada");
        let destination = nested.join("..").join("destino.txt");

        let normalized = validate_destination(None, &destination, DestinationPolicy::default())
            .expect("o destino deve ser normalizado");
        let expected = fs::canonicalize(&root)
            .expect("a raiz do teste deve ser normalizável")
            .join("destino.txt");
        assert_eq!(normalized, expected);
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }

    #[test]
    fn detecta_mesma_origem_mesmo_com_caminho_equivalente() {
        let root = temporary_directory();
        let nested = root.join("nested");
        let source = root.join("origem.txt");
        fs::create_dir(&nested).expect("a pasta aninhada deve ser criada");
        fs::write(&source, b"conteudo").expect("a origem deve ser criada");
        let equivalent_destination = nested.join("..").join("origem.txt");

        let result = validate_destination(
            Some(&source),
            &equivalent_destination,
            DestinationPolicy::default(),
        );
        assert!(matches!(
            result,
            Err(ValidationError::SameSourceAndDestination { .. })
        ));
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }

    #[test]
    fn recusa_destino_relativo_com_mensagem_clara() {
        let result = validate_destination(
            None,
            std::path::Path::new("destino.txt"),
            DestinationPolicy::default(),
        );
        assert!(matches!(
            result,
            Err(ValidationError::InvalidPath {
                reason: "caminho relativo ambíguo; use um caminho absoluto",
                ..
            })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn recusa_componente_symlink_no_diretorio_pai() {
        use std::os::unix::fs::symlink;

        let root = temporary_directory();
        let outside = temporary_directory();
        let link = root.join("atalho");
        symlink(&outside, &link).expect("o symlink deve ser criado");
        let result =
            validate_destination(None, &link.join("novo.txt"), DestinationPolicy::default());
        assert!(matches!(result, Err(ValidationError::InvalidPath { .. })));
        fs::remove_dir_all(root).expect("a raiz do teste deve ser removida");
        fs::remove_dir_all(outside).expect("o destino externo deve ser removido");
    }

    #[test]
    fn recusa_componente_final_ambiguo() {
        let root = temporary_directory();
        let result = validate_destination(None, &root.join(".."), DestinationPolicy::default());
        assert!(matches!(result, Err(ValidationError::InvalidPath { .. })));
        fs::remove_dir_all(root).expect("o diretório deve ser removido");
    }
}
