//! Núcleo do Rovex.
//!
//! A biblioteca mantém a lógica de domínio separada da futura camada visual. O
//! binário de desenvolvimento usa esses contratos sem executar arquivos do
//! usuário ou depender de um shell.

pub mod filesystem;
pub mod operations;
pub mod security;

pub use filesystem::{DirectoryEntry, EntryKind, FileSystem, FileSystemError};
pub use operations::{
    copy_file_atomic, create_directory, delete_entry, rename_entry, CopyReport, OperationError,
};
pub use security::{validate_destination, DestinationPolicy};
