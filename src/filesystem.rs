use std::ffi::{OsStr, OsString};
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
    pub created: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub is_hidden: bool,
    pub is_system: bool,
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

        let is_hidden = is_hidden_name_and_metadata(&name, &metadata);
        let is_system = is_system_metadata(&metadata);
        Ok(Self {
            path,
            name,
            kind,
            size: (kind == EntryKind::File).then_some(metadata.len()),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            accessed: metadata.accessed().ok(),
            is_hidden,
            is_system,
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
                ..
            } => write!(
                formatter,
                "não foi possível {} em {}: {}",
                operation,
                path.display(),
                human_io_reason(*kind)
            ),
        }
    }
}

impl std::error::Error for FileSystemError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListingOptions {
    pub show_hidden: bool,
    pub show_system: bool,
}

fn is_hidden_name_and_metadata(name: &OsStr, metadata: &fs::Metadata) -> bool {
    let dot_hidden = name.to_string_lossy().starts_with('.');
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        return dot_hidden || metadata.file_attributes() & 0x2 != 0;
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        dot_hidden
    }
}

fn is_system_metadata(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        return metadata.file_attributes() & 0x4 != 0;
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileSystem;

impl FileSystem {
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirectoryEntry>, FileSystemError> {
        self.list_directory_with_options(path, ListingOptions::default())
    }

    pub fn list_directory_with_options(
        &self,
        path: &Path,
        options: ListingOptions,
    ) -> Result<Vec<DirectoryEntry>, FileSystemError> {
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
            let directory_entry = DirectoryEntry::from_path(entry.path(), entry.file_name())?;
            if (!options.show_hidden && directory_entry.is_hidden)
                || (!options.show_system && directory_entry.is_system)
            {
                continue;
            }
            entries.push(directory_entry);
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
mod tests;
