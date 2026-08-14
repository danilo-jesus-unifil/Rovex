use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub name: OsString,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

impl DirectoryEntry {
    fn from_path(path: PathBuf, name: OsString) -> Result<Self, FileSystemError> {
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| FileSystemError::from_io("ler metadados", &path, error))?;
        let file_type = metadata.file_type();
        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        Ok(Self {
            path,
            name,
            kind,
            size: (kind == EntryKind::File).then_some(metadata.len()),
            modified: metadata.modified().ok(),
        })
    }

    pub fn display_name(&self) -> String {
        self.name.to_string_lossy().into_owned()
    }
}

#[derive(Debug)]
pub enum FileSystemError {
    NotFound {
        path: PathBuf,
    },
    NotDirectory {
        path: PathBuf,
    },
    AccessDenied {
        path: PathBuf,
    },
    InvalidPath {
        path: PathBuf,
        reason: &'static str,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        kind: io::ErrorKind,
        raw_os_error: Option<i32>,
    },
}

impl FileSystemError {
    pub(crate) fn from_io(operation: &'static str, path: &Path, error: io::Error) -> Self {
        let path = path.to_path_buf();
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path },
            io::ErrorKind::PermissionDenied => Self::AccessDenied { path },
            kind => Self::Io {
                operation,
                path,
                kind,
                raw_os_error: error.raw_os_error(),
            },
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { path } => {
                write!(formatter, "caminho não encontrado: {}", path.display())
            }
            Self::NotDirectory { path } => {
                write!(formatter, "não é um diretório: {}", path.display())
            }
            Self::AccessDenied { path } => write!(formatter, "acesso negado: {}", path.display()),
            Self::InvalidPath { path, reason } => {
                write!(
                    formatter,
                    "caminho inválido ({}): {}",
                    reason,
                    path.display()
                )
            }
            Self::Io {
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
        }
    }
}

impl std::error::Error for FileSystemError {}

#[derive(Debug, Clone, Copy)]
pub struct FileSystem;

impl FileSystem {
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirectoryEntry>, FileSystemError> {
        if path.as_os_str().is_empty() {
            return Err(FileSystemError::InvalidPath {
                path: path.to_path_buf(),
                reason: "vazio",
            });
        }

        let metadata = fs::symlink_metadata(path)
            .map_err(|error| FileSystemError::from_io("ler diretório", path, error))?;
        if !metadata.is_dir() {
            return Err(FileSystemError::NotDirectory {
                path: path.to_path_buf(),
            });
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(path)
            .map_err(|error| FileSystemError::from_io("listar diretório", path, error))?
        {
            let entry =
                entry.map_err(|error| FileSystemError::from_io("ler entrada", path, error))?;
            entries.push(DirectoryEntry::from_path(entry.path(), entry.file_name())?);
        }

        entries.sort_by_cached_key(|entry| {
            (
                !matches!(entry.kind, EntryKind::Directory),
                entry.display_name().to_lowercase(),
            )
        });
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::{EntryKind, FileSystem};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("o relógio do sistema deve estar depois da época Unix")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rovex-filesystem-test-{unique}"));
        fs::create_dir(&path).expect("o diretório temporário deve ser criado");
        path
    }

    #[test]
    fn lista_diretorios_antes_de_arquivos() {
        let root = temporary_directory();
        fs::create_dir(root.join("Pasta")).expect("a pasta deve ser criada");
        fs::write(root.join("arquivo.txt"), b"conteudo").expect("o arquivo deve ser criado");

        let entries = FileSystem
            .list_directory(&root)
            .expect("a listagem deve funcionar");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, EntryKind::Directory);
        assert_eq!(entries[1].kind, EntryKind::File);
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[test]
    fn rejeita_caminho_que_nao_e_diretorio() {
        let root = temporary_directory();
        let file = root.join("arquivo.txt");
        fs::write(&file, b"conteudo").expect("o arquivo deve ser criado");

        let result = FileSystem.list_directory(&file);
        assert!(matches!(
            result,
            Err(super::FileSystemError::NotDirectory { .. })
        ));
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }
}
