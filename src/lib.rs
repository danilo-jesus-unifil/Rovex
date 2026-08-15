//! Núcleo do Rovex.
//!
//! A biblioteca mantém a lógica de domínio separada da futura camada visual. O
//! binário de desenvolvimento usa esses contratos sem executar arquivos do
//! usuário ou depender de um shell.

pub mod desktop;
pub mod filesystem;
pub mod operations;
pub mod security;

pub use filesystem::{DirectoryEntry, EntryKind, FileSystem, FileSystemError};
pub use operations::{
    copy_file_atomic, copy_file_atomic_with_progress, create_directory, delete_entry, rename_entry,
    CopyProgress, CopyReport, OperationError,
};
pub use security::{validate_destination, DestinationPolicy};
