//! Conversões reais usando executáveis FFmpeg instalados pelo sistema.
//!
//! O ambiente de desenvolvimento foi validado com FFmpeg/ffprobe 6.1.1 do
//! Ubuntu 24.04, incluindo os codecs libjxl, libopus, PNG e FLAC. No Windows,
//! `ffmpeg.exe` e `ffprobe.exe` devem estar disponíveis no `PATH`; o Rovex não
//! baixa executáveis nem invoca shell em runtime.

use crate::operations::{OperationError, publish_file_no_replace};
use crate::security::{DestinationPolicy, ValidationError, validate_destination, validate_source};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_CONVERSION_DURATION: Duration = Duration::from_secs(5 * 60);

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
            Self::BackendUnavailable { executable } => {
                write!(
                    formatter,
                    "o conversor `{executable}` não foi encontrado no PATH"
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

fn absolute_source(source: &Path) -> Result<PathBuf, ConversionError> {
    if source.is_absolute() {
        return Ok(source.to_path_buf());
    }
    std::env::current_dir()
        .map(|directory| directory.join(source))
        .map_err(|_| ConversionError::InvalidInput {
            path: source.to_path_buf(),
            reason: "não foi possível determinar o diretório atual",
        })
}

fn output_path(source: &Path, kind: ConversionKind) -> Result<PathBuf, ConversionError> {
    let source = absolute_source(source)?;
    let parent = source
        .parent()
        .ok_or_else(|| ConversionError::InvalidInput {
            path: source.clone(),
            reason: "o arquivo não possui diretório pai",
        })?;
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ConversionError::InvalidInput {
            path: source.clone(),
            reason: "o arquivo não possui nome compatível",
        })?;
    let extension = kind.extension();
    let mut destination = parent.join(format!("{stem}.{extension}"));
    if destination == source {
        destination = parent.join(format!("{stem}.converted.{extension}"));
    }
    Ok(destination)
}

fn temporary_path(destination: &Path) -> Result<PathBuf, ConversionError> {
    let parent = destination
        .parent()
        .ok_or_else(|| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "a saída não possui diretório pai",
        })?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "a saída não possui nome válido",
        })?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ConversionError::InvalidInput {
            path: destination.to_path_buf(),
            reason: "o relógio do sistema não pôde ser lido",
        })?
        .as_nanos();
    for attempt in 0..32_u32 {
        let candidate = parent.join(format!(
            ".{name}.rovex-convert-{}-{}",
            std::process::id(),
            timestamp + u128::from(attempt)
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ConversionError::InvalidInput {
        path: destination.to_path_buf(),
        reason: "não foi possível reservar um arquivo temporário",
    })
}

fn stderr_message(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim();
    if text.is_empty() {
        "o processo terminou sem diagnóstico".to_owned()
    } else {
        text.lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn spawn_ffmpeg(
    source: &Path,
    temporary: &Path,
    kind: ConversionKind,
    cancel: &AtomicBool,
    stage: &mut impl FnMut(ConversionStage),
) -> Result<(), ConversionError> {
    let mut command = Command::new("ffmpeg");
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-n")
        .arg("-i")
        .arg(source);
    match kind {
        ConversionKind::JpegXl => {
            command.args([
                "-frames:v",
                "1",
                "-c:v",
                "libjxl",
                "-distance",
                "1.0",
                "-f",
                "image2",
            ]);
        }
        ConversionKind::Png => {
            command.args(["-frames:v", "1", "-c:v", "png", "-f", "image2"]);
        }
        ConversionKind::Opus => {
            command.args([
                "-map", "0:a:0", "-vn", "-c:a", "libopus", "-b:a", "128k", "-f", "opus",
            ]);
        }
        ConversionKind::Flac => {
            command.args(["-map", "0:a:0", "-vn", "-c:a", "flac", "-f", "flac"]);
        }
    }
    stage(ConversionStage::Encoding);
    let mut child = command
        .arg(temporary)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                ConversionError::BackendUnavailable {
                    executable: "ffmpeg",
                }
            } else {
                ConversionError::Process {
                    executable: "ffmpeg",
                    path: source.to_path_buf(),
                    message: error.to_string(),
                }
            }
        })?;
    let deadline = Instant::now() + MAX_CONVERSION_DURATION;
    loop {
        if cancel.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Cancelled);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Timeout {
                executable: "ffmpeg",
                path: source.to_path_buf(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = child
                    .stderr
                    .take()
                    .and_then(|mut stderr| {
                        let mut bytes = Vec::new();
                        std::io::Read::read_to_end(&mut stderr, &mut bytes).ok()?;
                        Some(bytes)
                    })
                    .unwrap_or_default();
                if status.success() {
                    return Ok(());
                }
                return Err(ConversionError::Process {
                    executable: "ffmpeg",
                    path: source.to_path_buf(),
                    message: stderr_message(&stderr),
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(80)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ConversionError::Process {
                    executable: "ffmpeg",
                    path: source.to_path_buf(),
                    message: error.to_string(),
                });
            }
        }
    }
}

fn run_ffprobe(
    destination: &Path,
    stream: &str,
    cancel: &AtomicBool,
) -> Result<std::process::Output, ConversionError> {
    let mut child = Command::new("ffprobe")
        .arg("-hide_banner")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg(stream)
        .arg("-show_entries")
        .arg("stream=codec_name")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(destination)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                ConversionError::BackendUnavailable {
                    executable: "ffprobe",
                }
            } else {
                ConversionError::Process {
                    executable: "ffprobe",
                    path: destination.to_path_buf(),
                    message: error.to_string(),
                }
            }
        })?;
    let deadline = Instant::now() + MAX_CONVERSION_DURATION;
    loop {
        if cancel.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Cancelled);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Timeout {
                executable: "ffprobe",
                path: destination.to_path_buf(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                if let Some(mut pipe) = child.stdout.take() {
                    std::io::Read::read_to_end(&mut pipe, &mut stdout).map_err(|error| {
                        ConversionError::Process {
                            executable: "ffprobe",
                            path: destination.to_path_buf(),
                            message: error.to_string(),
                        }
                    })?;
                }
                let mut stderr = Vec::new();
                if let Some(mut pipe) = child.stderr.take() {
                    std::io::Read::read_to_end(&mut pipe, &mut stderr).map_err(|error| {
                        ConversionError::Process {
                            executable: "ffprobe",
                            path: destination.to_path_buf(),
                            message: error.to_string(),
                        }
                    })?;
                }
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ConversionError::Process {
                    executable: "ffprobe",
                    path: destination.to_path_buf(),
                    message: error.to_string(),
                });
            }
        }
    }
}

fn validate_output(
    destination: &Path,
    kind: ConversionKind,
    cancel: &AtomicBool,
) -> Result<(), ConversionError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ConversionError::Cancelled);
    }
    let output_metadata =
        fs::symlink_metadata(destination).map_err(|error| ConversionError::Process {
            executable: "ffprobe",
            path: destination.to_path_buf(),
            message: error.to_string(),
        })?;
    if !output_metadata.file_type().is_file() || output_metadata.len() == 0 {
        return Err(ConversionError::OutputValidationFailed {
            path: destination.to_path_buf(),
            expected_codec: kind.expected_codec(),
            detected_codec: "saída ausente ou vazia".to_owned(),
        });
    }
    let stream = match kind {
        ConversionKind::JpegXl | ConversionKind::Png => "v:0",
        ConversionKind::Opus | ConversionKind::Flac => "a:0",
    };
    let output = run_ffprobe(destination, stream, cancel)?;
    let detected = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || detected != kind.expected_codec() {
        return Err(ConversionError::OutputValidationFailed {
            path: destination.to_path_buf(),
            expected_codec: kind.expected_codec(),
            detected_codec: if detected.is_empty() {
                stderr_message(&output.stderr)
            } else {
                detected
            },
        });
    }
    Ok(())
}

fn map_destination_error(error: OperationError, destination: &Path) -> ConversionError {
    match &error {
        OperationError::Validation(ValidationError::ExistingDestination { .. })
        | OperationError::FileSystem {
            kind: io::ErrorKind::AlreadyExists,
            ..
        } => ConversionError::OutputExists {
            path: destination.to_path_buf(),
        },
        _ => ConversionError::Operation(error),
    }
}

pub fn convert_file<F>(
    source: &Path,
    kind: ConversionKind,
    cancel: &AtomicBool,
    mut stage: F,
) -> Result<ConversionReport, ConversionError>
where
    F: FnMut(ConversionStage),
{
    let source_type =
        validate_source(source).map_err(|error| ConversionError::Operation(error.into()))?;
    if !source_type.is_file() || source_type.is_symlink() {
        return Err(ConversionError::InvalidInput {
            path: source.to_path_buf(),
            reason: "a conversão exige um arquivo regular; links simbólicos não são seguidos",
        });
    }
    let source = absolute_source(source)?;
    if !kind.accepts(&source) {
        return Err(ConversionError::InvalidInput {
            path: source,
            reason: "a extensão não corresponde ao conversor escolhido",
        });
    }
    if cancel.load(Ordering::Acquire) {
        return Err(ConversionError::Cancelled);
    }
    stage(ConversionStage::Starting);
    let destination = output_path(&source, kind)?;
    let destination =
        validate_destination(Some(&source), &destination, DestinationPolicy::default()).map_err(
            |error| map_destination_error(OperationError::Validation(error), &destination),
        )?;
    let temporary = temporary_path(&destination)?;
    let result = (|| {
        spawn_ffmpeg(&source, &temporary, kind, cancel, &mut stage)?;
        stage(ConversionStage::Validating);
        validate_output(&temporary, kind, cancel)?;
        if cancel.load(Ordering::Acquire) {
            return Err(ConversionError::Cancelled);
        }
        stage(ConversionStage::Publishing);
        publish_file_no_replace(&temporary, &destination)
            .map_err(|error| map_destination_error(error, &destination))?;
        Ok(ConversionReport {
            source: source.clone(),
            destination: destination.clone(),
            codec: kind.expected_codec(),
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{ConversionKind, convert_file, output_path};
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn conversores_reconhecem_extensoes_sem_diferenciar_maiusculas() {
        assert!(ConversionKind::JpegXl.accepts(Path::new("foto.JPEG")));
        assert!(ConversionKind::Opus.accepts(Path::new("faixa.WAV")));
        assert!(!ConversionKind::Opus.accepts(Path::new("foto.jpg")));
    }

    #[test]
    fn saida_usa_nome_irmao_e_evita_mesmo_caminho() {
        let jxl = output_path(Path::new("/tmp/foto.jpg"), ConversionKind::JpegXl).unwrap();
        assert_eq!(jxl, Path::new("/tmp/foto.jxl"));
        let same = output_path(Path::new("/tmp/foto.jxl"), ConversionKind::JpegXl).unwrap();
        assert_eq!(same, Path::new("/tmp/foto.converted.jxl"));
    }

    #[test]
    fn cancelamento_antes_do_backend_nao_publica_saida() {
        let source = std::env::temp_dir().join(format!(
            "rovex-converter-cancel-{}-{}.png",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&source, b"entrada de teste").expect("criar origem temporária");
        let cancel = AtomicBool::new(true);
        let error = convert_file(&source, ConversionKind::JpegXl, &cancel, |_| {})
            .expect_err("cancelamento deve impedir conversão");
        assert!(matches!(error, super::ConversionError::Cancelled));
        assert!(!source.with_extension("jxl").exists());
        let _ = fs::remove_file(source);
    }

    #[test]
    #[ignore = "requer FFmpeg e ffprobe instalados no ambiente"]
    fn conversoes_reais_publicam_saidas_validadas_pelo_ffprobe() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio monotônico disponível")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "rovex-converter-test-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("criar diretório temporário");
        let image = directory.join("entrada.png");
        let audio = directory.join("entrada.wav");
        let create_image = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=8x8",
                "-frames:v",
                "1",
            ])
            .arg(&image)
            .status()
            .expect("executar ffmpeg para a imagem");
        assert!(create_image.success());
        let create_audio = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.15",
                "-c:a",
                "pcm_s16le",
            ])
            .arg(&audio)
            .status()
            .expect("executar ffmpeg para o áudio");
        assert!(create_audio.success());

        let cancel = AtomicBool::new(false);
        let jxl = convert_file(&image, ConversionKind::JpegXl, &cancel, |_| {})
            .expect("converter imagem para JXL");
        assert!(jxl.destination.is_file());
        assert!(fs::metadata(&jxl.destination).unwrap().len() > 0);
        let png = convert_file(&image, ConversionKind::Png, &cancel, |_| {})
            .expect("converter imagem para PNG");
        assert!(png.destination.is_file());
        let opus = convert_file(&audio, ConversionKind::Opus, &cancel, |_| {})
            .expect("converter áudio para Opus");
        assert!(opus.destination.is_file());
        let flac = convert_file(&audio, ConversionKind::Flac, &cancel, |_| {})
            .expect("converter áudio para FLAC");
        assert!(flac.destination.is_file());

        let second = convert_file(&image, ConversionKind::JpegXl, &cancel, |_| {})
            .expect_err("recusar saída JXL já existente");
        assert!(matches!(
            second,
            super::ConversionError::OutputExists { .. }
        ));
        let _ = fs::remove_dir_all(&directory);
    }
}
