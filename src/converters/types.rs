use crate::operations::OperationError;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionKind {
    JpegXl,
    Opus,
    Png,
    Flac,
}

impl ConversionKind {
    pub fn extension(self) -> &'static str {
        match self {
            Self::JpegXl => "jxl",
            Self::Opus => "opus",
            Self::Png => "png",
            Self::Flac => "flac",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::JpegXl => "JPEG XL",
            Self::Opus => "Opus",
            Self::Png => "PNG",
            Self::Flac => "FLAC",
        }
    }

    pub fn expected_codec(self) -> &'static str {
        match self {
            Self::JpegXl => "jpegxl",
            Self::Opus => "opus",
            Self::Png => "png",
            Self::Flac => "flac",
        }
    }

    pub fn accepts(self, path: &Path) -> bool {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            return false;
        };
        let extension = extension.to_ascii_lowercase();
        match self {
            Self::JpegXl | Self::Png => matches!(
                extension.as_str(),
                "avif"
                    | "bmp"
                    | "gif"
                    | "heic"
                    | "jpeg"
                    | "jpg"
                    | "jxl"
                    | "png"
                    | "tif"
                    | "tiff"
                    | "webp"
            ),
            Self::Opus | Self::Flac => matches!(
                extension.as_str(),
                "aac"
                    | "flac"
                    | "m4a"
                    | "mka"
                    | "mp3"
                    | "mp4"
                    | "oga"
                    | "ogg"
                    | "opus"
                    | "wav"
                    | "webm"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionStage {
    Starting,
    Encoding,
    Validating,
    Publishing,
}

impl ConversionStage {
    pub fn percent(self) -> u8 {
        match self {
            Self::Starting => 5,
            Self::Encoding => 55,
            Self::Validating => 85,
            Self::Publishing => 95,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionReport {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub codec: &'static str,
}

#[derive(Debug)]
pub enum ConversionError {
    BackendUnavailable {
        executable: &'static str,
        attempts: usize,
    },
    InvalidInput {
        path: PathBuf,
        reason: &'static str,
    },
    OutputExists {
        path: PathBuf,
    },
    Process {
        executable: &'static str,
        path: PathBuf,
        message: String,
    },
    Timeout {
        executable: &'static str,
        path: PathBuf,
    },
    OutputValidationFailed {
        path: PathBuf,
        expected_codec: &'static str,
        detected_codec: String,
    },
    Operation(OperationError),
    Cancelled,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BackendUnavailable {
                executable,
                attempts,
            } => {
                write!(
                    formatter,
                    "o conversor `{executable}` não foi encontrado; foram tentadas {attempts} localizações seguras (PATH, diretório do Rovex e diretórios padrão). Defina ROVEX_{executable_upper}_PATH com o caminho absoluto do executável, se necessário",
                    executable_upper = executable.to_ascii_uppercase(),
                )
            }
            Self::InvalidInput { path, reason } => {
                write!(
                    formatter,
                    "entrada de conversão inválida ({reason}): {}",
                    path.display()
                )
            }
            Self::OutputExists { path } => {
                write!(
                    formatter,
                    "o arquivo de saída já existe: {}",
                    path.display()
                )
            }
            Self::Process {
                executable,
                path,
                message,
            } => write!(
                formatter,
                "`{executable}` não conseguiu converter {}: {message}",
                path.display()
            ),
            Self::Timeout { executable, path } => write!(
                formatter,
                "`{executable}` excedeu o limite de cinco minutos ao processar {}",
                path.display()
            ),
            Self::OutputValidationFailed {
                path,
                expected_codec,
                detected_codec,
            } => write!(
                formatter,
                "a saída {} não foi validada como {expected_codec} (detectado: {detected_codec})",
                path.display()
            ),
            Self::Operation(error) => error.fmt(formatter),
            Self::Cancelled => write!(formatter, "conversão cancelada pelo usuário"),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<OperationError> for ConversionError {
    fn from(error: OperationError) -> Self {
        Self::Operation(error)
    }
}
