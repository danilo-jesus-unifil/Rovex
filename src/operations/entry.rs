use super::error::{OperationError, from_io};
use crate::security::{
    DestinationPolicy, ValidationError, ensure_not_root, validate_destination, validate_source,
};
use std::fs;
#[cfg(not(windows))]
use std::io;
use std::path::Path;

fn is_reparse_point(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        fs::symlink_metadata(path)
            .map(|metadata| metadata.file_attributes() & 0x400 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
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

#[cfg(not(windows))]
fn is_directory_not_empty(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::DirectoryNotEmpty
        || matches!(error.raw_os_error(), Some(39) | Some(145))
}

#[cfg(windows)]
fn ensure_directory_empty(path: &Path) -> Result<(), OperationError> {
    let mut entries = fs::read_dir(path).map_err(|error| from_io("ler diretório", path, error))?;
    if entries
        .next()
        .transpose()
        .map_err(|error| from_io("ler entrada", path, error))?
        .is_some()
    {
        return Err(OperationError::DirectoryNotEmpty {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

pub fn delete_entry(path: &Path) -> Result<(), OperationError> {
    ensure_not_root(path)?;
    let file_type = validate_source(path)?;
    if file_type.is_symlink() || file_type.is_file() || is_reparse_point(path) {
        #[cfg(windows)]
        {
            return super::recycle::delete_to_recycle_bin(path);
        }
        #[cfg(not(windows))]
        {
            fs::remove_file(path).map_err(|error| from_io("excluir entrada", path, error))?;
            return Ok(());
        }
    }

    if file_type.is_dir() {
        #[cfg(windows)]
        {
            ensure_directory_empty(path)?;
            super::recycle::delete_to_recycle_bin(path)
        }
        #[cfg(not(windows))]
        {
            match fs::remove_dir(path) {
                Ok(()) => Ok(()),
                Err(error) if is_directory_not_empty(&error) => {
                    Err(OperationError::DirectoryNotEmpty {
                        path: path.to_path_buf(),
                    })
                }
                Err(error) => Err(from_io("excluir diretório", path, error)),
            }
        }
    } else {
        Err(OperationError::Validation(
            ValidationError::UnsupportedSource {
                path: path.to_path_buf(),
            },
        ))
    }
}
