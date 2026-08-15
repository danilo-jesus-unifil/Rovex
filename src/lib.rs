//! Núcleo do Rovex.
//!
//! A biblioteca mantém a lógica de domínio separada da futura camada visual. O
//! binário de desenvolvimento usa esses contratos sem executar arquivos do
//! usuário ou depender de um shell.

pub mod converters;
pub mod desktop;
pub mod filesystem;
pub mod operations;
pub mod security;

pub use converters::{
    ConversionError, ConversionKind, ConversionReport, ConversionStage, convert_file,
};
pub use filesystem::{DirectoryEntry, EntryKind, FileSystem, FileSystemError};
pub use operations::{
    CopyProgress, CopyReport, OperationError, copy_file_atomic, copy_file_atomic_with_progress,
    create_directory, delete_entry, rename_entry,
};
pub use security::{DestinationPolicy, validate_destination};
