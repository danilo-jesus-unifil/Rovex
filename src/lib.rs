//! Núcleo do Rovex.
//!
//! A biblioteca mantém a lógica de domínio separada da futura camada visual. O
//! binário de desenvolvimento usa esses contratos sem executar arquivos do
//! usuário ou depender de um shell.

pub mod clipboard;
pub mod converters;
pub mod desktop;
pub mod filesystem;
pub mod operations;
pub mod preview;
pub mod search;
pub mod security;
pub mod settings;
mod terminal;

pub use converters::{
    ConversionError, ConversionKind, ConversionReport, ConversionStage, convert_file,
};
pub use filesystem::{DirectoryEntry, EntryKind, FileSystem, FileSystemError};
pub use operations::{
    CopyProgress, CopyReport, OperationError, copy_file_atomic, copy_file_atomic_with_progress,
    create_directory, delete_entry, rename_entry,
};
pub use preview::{PreviewError, PreviewImage, PreviewLimits, decode_thumbnail};
pub use search::{
    SearchError, SearchLimit, SearchLimits, SearchReport, SearchStatus, SearchUpdate,
    search_by_name,
};
pub use security::{DestinationPolicy, validate_destination};
pub use terminal::{
    TerminalError, is_supported as terminal_supported, open_terminal_for_item, open_terminal_here,
    target_directory,
};
