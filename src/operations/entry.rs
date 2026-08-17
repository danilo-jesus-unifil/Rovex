use super::error::{OperationError, from_io};
use crate::security::{
    DestinationPolicy, ValidationError, ensure_not_root, validate_destination, validate_source,
};
use std::fs;
use std::io;
use std::path::Path;

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
    error.kind() == io::ErrorKind::DirectoryNotEmpty
        || matches!(error.raw_os_error(), Some(39) | Some(145))
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
