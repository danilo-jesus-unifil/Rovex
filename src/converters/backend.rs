use super::types::ConversionError;
#[cfg(windows)]
use super::windows_backend::{
    windows_app_path_entries, windows_persistent_path_entries, windows_winget_package_candidates,
};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendResolution {
    pub(crate) path: PathBuf,
}

fn backend_override_name(executable: &str) -> String {
    format!("ROVEX_{}_PATH", executable.to_ascii_uppercase())
}

fn backend_names(executable: &str) -> [PathBuf; 2] {
    [
        PathBuf::from(executable),
        PathBuf::from(format!("{executable}.exe")),
    ]
}

pub(crate) fn push_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidate.as_os_str().is_empty() && !candidates.iter().any(|path| path == &candidate) {
        candidates.push(candidate);
    }
}

pub(crate) fn push_directory_candidates(
    candidates: &mut Vec<PathBuf>,
    directory: PathBuf,
    executable: &str,
) {
    for name in backend_names(executable) {
        push_candidate(candidates, directory.join(name));
    }
}

pub(crate) fn push_path_or_directory_candidates(
    candidates: &mut Vec<PathBuf>,
    path: PathBuf,
    executable: &str,
) {
    push_candidate(candidates, path.clone());
    if path.extension().is_none() {
        push_candidate(candidates, path.with_extension("exe"));
    }
    push_directory_candidates(candidates, path, executable);
}
pub(crate) fn backend_candidates(
    executable: &str,
    adjacent_directory: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(override_path) = std::env::var_os(backend_override_name(executable)) {
        let override_path = PathBuf::from(override_path);
        if override_path.is_absolute() {
            push_path_or_directory_candidates(&mut candidates, override_path, executable);
        }
    }

    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            push_directory_candidates(&mut candidates, directory, executable);
        }
    }

    #[cfg(windows)]
    {
        for directory in windows_persistent_path_entries() {
            push_directory_candidates(&mut candidates, directory, executable);
        }
        for candidate in windows_app_path_entries(executable) {
            push_candidate(&mut candidates, candidate);
        }
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(directory) = current_exe.parent()
    {
        push_directory_candidates(&mut candidates, directory.to_path_buf(), executable);
    }
    // Não adicionar o CWD implicitamente: no Windows ele pode preceder o PATH
    // conforme SafeProcessSearchMode e permitir plantio acidental de binário.
    if let Some(directory) = adjacent_directory {
        push_directory_candidates(&mut candidates, directory.to_path_buf(), executable);
    }

    if let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        push_directory_candidates(&mut candidates, home.join(".local").join("bin"), executable);
        push_directory_candidates(&mut candidates, home.join("bin"), executable);
        push_directory_candidates(
            &mut candidates,
            home.join("scoop").join("shims"),
            executable,
        );
        push_directory_candidates(
            &mut candidates,
            home.join("scoop")
                .join("apps")
                .join("ffmpeg")
                .join("current")
                .join("bin"),
            executable,
        );
        push_directory_candidates(
            &mut candidates,
            home.join("AppData")
                .join("Local")
                .join("Programs")
                .join("ffmpeg")
                .join("bin"),
            executable,
        );
        #[cfg(windows)]
        windows_winget_package_candidates(
            &mut candidates,
            &home
                .join("AppData")
                .join("Local")
                .join("Microsoft")
                .join("WinGet")
                .join("Packages"),
            executable,
        );
    }

    for variable in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "LOCALAPPDATA",
        "ChocolateyToolsLocation",
        "ChocolateyInstall",
    ] {
        if let Some(root) = std::env::var_os(variable).map(PathBuf::from) {
            push_directory_candidates(&mut candidates, root.join("ffmpeg").join("bin"), executable);
            push_directory_candidates(&mut candidates, root.join("FFmpeg").join("bin"), executable);
            push_directory_candidates(
                &mut candidates,
                root.join("ffmpeg").join("current").join("bin"),
                executable,
            );
            push_directory_candidates(&mut candidates, root.join("bin"), executable);
            push_directory_candidates(
                &mut candidates,
                root.join("lib")
                    .join("ffmpeg")
                    .join("tools")
                    .join("ffmpeg")
                    .join("bin"),
                executable,
            );
            push_directory_candidates(
                &mut candidates,
                root.join("Microsoft").join("WinGet").join("Links"),
                executable,
            );
            push_directory_candidates(
                &mut candidates,
                root.join("WinGet").join("Links"),
                executable,
            );
            #[cfg(windows)]
            {
                windows_winget_package_candidates(
                    &mut candidates,
                    &root.join("Microsoft").join("WinGet").join("Packages"),
                    executable,
                );
                windows_winget_package_candidates(
                    &mut candidates,
                    &root.join("WinGet").join("Packages"),
                    executable,
                );
            }
        }
    }

    for variable in ["FFMPEG_HOME", "FFMPEG_ROOT", "FFMPEG_DIR", "FFMPEG_PATH"] {
        if let Some(path) = std::env::var_os(variable).map(PathBuf::from)
            && path.is_absolute()
        {
            push_path_or_directory_candidates(&mut candidates, path, executable);
        }
    }

    for directory in [
        "/usr/bin",
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/snap/bin",
        "/var/lib/flatpak/exports/bin",
        "C:\\ffmpeg\\bin",
        "C:\\ProgramData\\chocolatey\\bin",
    ] {
        push_directory_candidates(&mut candidates, PathBuf::from(directory), executable);
    }

    candidates
}

pub(crate) fn is_backend_file(path: &Path) -> bool {
    // O worker recebe apenas caminhos absolutos. `metadata` segue symlinks e
    // junctions, necessários para instalações via gerenciadores de pacotes e
    // links do sistema; continua recusando diretórios.
    path.is_absolute()
        && fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
}

#[cfg(test)]
pub(crate) fn resolve_backend_from_candidates(
    executable: &'static str,
    candidates: &[PathBuf],
) -> Result<BackendResolution, ConversionError> {
    for candidate in candidates {
        if is_backend_file(candidate) {
            return Ok(BackendResolution {
                path: candidate.clone(),
            });
        }
    }
    Err(ConversionError::BackendUnavailable {
        executable,
        attempts: candidates.len(),
    })
}

#[cfg(test)]
pub(crate) fn resolve_backend_with_adjacent(
    executable: &'static str,
    adjacent_directory: Option<&Path>,
) -> Result<BackendResolution, ConversionError> {
    let candidates = backend_candidates(executable, adjacent_directory);
    resolve_backend_from_candidates(executable, &candidates)
}

#[cfg(test)]
pub(crate) fn resolve_backend(
    executable: &'static str,
) -> Result<BackendResolution, ConversionError> {
    resolve_backend_with_adjacent(executable, None)
}

pub(crate) fn is_backend_retryable_error(error: &ConversionError) -> bool {
    matches!(
        error,
        ConversionError::BackendUnavailable { .. }
            | ConversionError::Process { .. }
            | ConversionError::Timeout { .. }
            | ConversionError::OutputValidationFailed { .. }
    )
}
