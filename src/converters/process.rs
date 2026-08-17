use super::process_output::stderr_message;
use super::process_output::{join_output_reader, start_output_reader};
use super::types::{ConversionError, ConversionKind, ConversionStage};
use crate::operations::OperationError;
use crate::security::ValidationError;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const MAX_CONVERSION_DURATION: Duration = Duration::from_secs(5 * 60);

pub(crate) fn spawn_ffmpeg(
    backend: &Path,
    source: &Path,
    temporary: &Path,
    kind: ConversionKind,
    cancel: &AtomicBool,
    stage: &mut impl FnMut(ConversionStage),
) -> Result<(), ConversionError> {
    let mut command = Command::new(backend);
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
        .map_err(|error| ConversionError::Process {
            executable: "ffmpeg",
            path: source.to_path_buf(),
            message: format!("o executável resolvido não pôde ser iniciado: {error}"),
        })?;
    let mut stderr_reader = match start_output_reader(child.stderr.take(), "rovex-ffmpeg-stderr") {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Process {
                executable: "ffmpeg",
                path: source.to_path_buf(),
                message: format!("não foi possível ler o diagnóstico do processo: {error}"),
            });
        }
    };
    let deadline = Instant::now() + MAX_CONVERSION_DURATION;
    loop {
        if cancel.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_output_reader(stderr_reader.take());
            return Err(ConversionError::Cancelled);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_output_reader(stderr_reader.take());
            return Err(ConversionError::Timeout {
                executable: "ffmpeg",
                path: source.to_path_buf(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = join_output_reader(stderr_reader.take()).map_err(|error| {
                    ConversionError::Process {
                        executable: "ffmpeg",
                        path: source.to_path_buf(),
                        message: error.to_string(),
                    }
                })?;
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
                let _ = join_output_reader(stderr_reader.take());
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
    backend: &Path,
    destination: &Path,
    stream: &str,
    cancel: &AtomicBool,
) -> Result<std::process::Output, ConversionError> {
    let mut child = Command::new(backend)
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
        .map_err(|error| ConversionError::Process {
            executable: "ffprobe",
            path: destination.to_path_buf(),
            message: format!("o executável resolvido não pôde ser iniciado: {error}"),
        })?;
    let mut stdout_reader = match start_output_reader(child.stdout.take(), "rovex-ffprobe-stdout") {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ConversionError::Process {
                executable: "ffprobe",
                path: destination.to_path_buf(),
                message: format!("não foi possível ler a saída do processo: {error}"),
            });
        }
    };
    let mut stderr_reader = match start_output_reader(child.stderr.take(), "rovex-ffprobe-stderr") {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_output_reader(stdout_reader.take());
            return Err(ConversionError::Process {
                executable: "ffprobe",
                path: destination.to_path_buf(),
                message: format!("não foi possível ler o diagnóstico do processo: {error}"),
            });
        }
    };
    let deadline = Instant::now() + MAX_CONVERSION_DURATION;
    loop {
        if cancel.load(Ordering::Acquire) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_output_reader(stdout_reader.take());
            let _ = join_output_reader(stderr_reader.take());
            return Err(ConversionError::Cancelled);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = join_output_reader(stdout_reader.take());
            let _ = join_output_reader(stderr_reader.take());
            return Err(ConversionError::Timeout {
                executable: "ffprobe",
                path: destination.to_path_buf(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = join_output_reader(stdout_reader.take()).map_err(|error| {
                    ConversionError::Process {
                        executable: "ffprobe",
                        path: destination.to_path_buf(),
                        message: error.to_string(),
                    }
                })?;
                let stderr = join_output_reader(stderr_reader.take()).map_err(|error| {
                    ConversionError::Process {
                        executable: "ffprobe",
                        path: destination.to_path_buf(),
                        message: error.to_string(),
                    }
                })?;
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
                let _ = join_output_reader(stdout_reader.take());
                let _ = join_output_reader(stderr_reader.take());
                return Err(ConversionError::Process {
                    executable: "ffprobe",
                    path: destination.to_path_buf(),
                    message: error.to_string(),
                });
            }
        }
    }
}

pub(crate) fn validate_output(
    ffprobe: &Path,
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
    let output = run_ffprobe(ffprobe, destination, stream, cancel)?;
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

pub(crate) fn map_destination_error(error: OperationError, destination: &Path) -> ConversionError {
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
