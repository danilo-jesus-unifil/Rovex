mod backend;
mod paths;
mod pipeline;
mod process;
mod process_output;
#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod tests;
mod types;
#[cfg(windows)]
mod windows_backend;

#[cfg(test)]
pub(crate) use backend::{
    backend_candidates, push_path_or_directory_candidates, resolve_backend,
    resolve_backend_from_candidates,
};
#[cfg(test)]
pub(crate) use paths::output_path;
pub use pipeline::convert_file;
#[cfg(test)]
pub(crate) use process_output::{MAX_PROCESS_OUTPUT_BYTES, read_limited_output};
pub use types::{ConversionError, ConversionKind, ConversionReport, ConversionStage};
