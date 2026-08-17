//! Conversões reais usando executáveis FFmpeg instalados pelo sistema.
//!
//! O ambiente de desenvolvimento foi validado com FFmpeg/ffprobe 6.1.1 do
//! Ubuntu 24.04, incluindo os codecs libjxl, libopus, PNG e FLAC. No Windows,
//! o Rovex tenta overrides absolutos, PATH herdado, PATH persistente, App Paths,
//! SearchPathW, diretório de trabalho, diretório do executável, variáveis FFMPEG_*
//! e locais seguros conhecidos; não baixa executáveis nem invoca shell em runtime.

use crate::operations::{OperationError, publish_file_no_replace};
use crate::security::{DestinationPolicy, ValidationError, validate_destination, validate_source};
#[cfg(windows)]
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct BackendResolution {
    path: PathBuf,
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

fn push_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidate.as_os_str().is_empty() && !candidates.iter().any(|path| path == &candidate) {
        candidates.push(candidate);
    }
}

fn push_directory_candidates(candidates: &mut Vec<PathBuf>, directory: PathBuf, executable: &str) {
    for name in backend_names(executable) {
        push_candidate(candidates, directory.join(name));
    }
}

fn push_path_or_directory_candidates(
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

#[cfg(windows)]
fn windows_wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
const MAX_REGISTRY_VALUE_BYTES: u32 = 1024 * 1024;

#[cfg(windows)]
fn expand_windows_environment(value: &str) -> Option<OsString> {
    use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;

    let source = windows_wide_null(value);
    // SAFETY: source é uma string UTF-16 terminada em NUL e o destino nulo apenas consulta o tamanho.
    let required = unsafe { ExpandEnvironmentStringsW(source.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 || required > MAX_REGISTRY_VALUE_BYTES / 2 {
        return None;
    }
    let mut expanded = vec![0u16; required as usize];
    // SAFETY: expanded tem espaço para o tamanho retornado pela API e o ponteiro de origem permanece válido.
    let written =
        unsafe { ExpandEnvironmentStringsW(source.as_ptr(), expanded.as_mut_ptr(), required) };
    if written == 0 {
        return None;
    }
    let length = (written as usize).saturating_sub(1).min(expanded.len());
    Some(OsString::from_wide(&expanded[..length]))
}

#[cfg(windows)]
fn windows_registry_access_modes() -> [u32; 3] {
    use windows_sys::Win32::System::Registry::{KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
    [
        KEY_READ,
        KEY_READ | KEY_WOW64_64KEY,
        KEY_READ | KEY_WOW64_32KEY,
    ]
}

#[cfg(windows)]
fn windows_registry_value(
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &str,
    value_name: Option<&str>,
    access: u32,
) -> Option<(u32, OsString)> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        REG_EXPAND_SZ, REG_SZ, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };

    let subkey = windows_wide_null(subkey);
    let value_name = value_name.map(windows_wide_null);
    let value_name_ptr = value_name
        .as_ref()
        .map_or(std::ptr::null(), |value| value.as_ptr());
    let mut key = std::ptr::null_mut();
    // SAFETY: as strings são terminadas em NUL, key é um ponteiro de saída válido e o acesso é somente leitura.
    let open_status = unsafe { RegOpenKeyExW(root, subkey.as_ptr(), 0, access, &mut key) };
    if open_status != ERROR_SUCCESS || key.is_null() {
        return None;
    }

    let mut value_type = 0;
    let mut byte_count = 0u32;
    // SAFETY: a primeira consulta pede apenas o tamanho e não escreve em dados.
    let size_status = unsafe {
        RegQueryValueExW(
            key,
            value_name_ptr,
            std::ptr::null(),
            &mut value_type,
            std::ptr::null_mut(),
            &mut byte_count,
        )
    };
    if size_status != ERROR_SUCCESS
        || !(2..=MAX_REGISTRY_VALUE_BYTES).contains(&byte_count)
        || (value_type != REG_SZ && value_type != REG_EXPAND_SZ)
    {
        // SAFETY: key foi aberto com sucesso nesta função e é fechado exatamente uma vez.
        unsafe { RegCloseKey(key) };
        return None;
    }

    let unit_count = (byte_count as usize).saturating_add(1) / 2;
    let mut buffer = vec![0u16; unit_count];
    // SAFETY: buffer tem espaço para o tamanho informado pela consulta anterior e o Windows escreve bytes UTF-16.
    let read_status = unsafe {
        RegQueryValueExW(
            key,
            value_name_ptr,
            std::ptr::null(),
            &mut value_type,
            buffer.as_mut_ptr().cast(),
            &mut byte_count,
        )
    };
    // SAFETY: key foi aberto com sucesso nesta função e é fechado exatamente uma vez.
    unsafe { RegCloseKey(key) };
    if read_status != ERROR_SUCCESS || byte_count > MAX_REGISTRY_VALUE_BYTES {
        return None;
    }

    let units = (byte_count as usize / 2).min(buffer.len());
    let raw = OsString::from_wide(&buffer[..units])
        .to_string_lossy()
        .trim_end_matches('\0')
        .to_owned();
    let value = if value_type == REG_EXPAND_SZ {
        expand_windows_environment(&raw).unwrap_or_else(|| OsString::from(raw))
    } else {
        OsString::from(raw)
    };
    Some((value_type, value))
}

#[cfg(windows)]
fn windows_persistent_path_entries() -> Vec<PathBuf> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    let mut entries = Vec::new();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for access in windows_registry_access_modes() {
            if let Some((_, value)) =
                windows_registry_value(root, "Environment", Some("Path"), access)
            {
                entries.extend(std::env::split_paths(&value));
            }
        }
    }
    entries
}

#[cfg(windows)]
fn windows_app_path_entries(executable: &str) -> Vec<PathBuf> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    let subkey =
        format!("Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{executable}.exe");
    let mut candidates = Vec::new();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for access in windows_registry_access_modes() {
            if let Some((_, value)) = windows_registry_value(root, &subkey, None, access) {
                let value = value.to_string_lossy();
                let value = value
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                    .unwrap_or(&value);
                push_path_or_directory_candidates(
                    &mut candidates,
                    PathBuf::from(value),
                    executable,
                );
            }
            if let Some((_, value)) = windows_registry_value(root, &subkey, Some("Path"), access) {
                for directory in std::env::split_paths(&value) {
                    push_directory_candidates(&mut candidates, directory, executable);
                }
            }
        }
    }
    candidates
}

#[cfg(windows)]
fn windows_search_path(executable: &str) -> Option<PathBuf> {
    use windows_sys::Win32::Storage::FileSystem::SearchPathW;

    let file_name = windows_wide_null(&format!("{executable}.exe"));
    let mut capacity = 260u32;
    for _ in 0..4 {
        let mut buffer = vec![0u16; capacity as usize];
        // SAFETY: file_name é NUL-terminated, buffer é gravável e filepart é opcional/nulo.
        let length = unsafe {
            SearchPathW(
                std::ptr::null(),
                file_name.as_ptr(),
                std::ptr::null(),
                buffer.len() as u32,
                buffer.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        };
        if length == 0 {
            return None;
        }
        if length < buffer.len() as u32 {
            return Some(PathBuf::from(OsString::from_wide(
                &buffer[..length as usize],
            )));
        }
        let next_capacity = length.saturating_add(1);
        if next_capacity <= capacity || next_capacity > 1024 * 1024 {
            return None;
        }
        capacity = next_capacity;
    }
    None
}

#[cfg(windows)]
fn windows_winget_package_candidates(
    candidates: &mut Vec<PathBuf>,
    packages_root: &Path,
    executable: &str,
) {
    const MAX_PACKAGES: usize = 64;
    const MAX_VERSIONS_PER_PACKAGE: usize = 16;
    let packages_root = packages_root.to_path_buf();
    let Ok(packages) = fs::read_dir(packages_root) else {
        return;
    };
    for package in packages.flatten().take(MAX_PACKAGES) {
        let package_path = package.path();
        let is_directory = package
            .file_type()
            .map(|kind| kind.is_dir())
            .unwrap_or(false);
        let name = package.file_name().to_string_lossy().to_ascii_lowercase();
        if !is_directory || !name.contains("ffmpeg") {
            continue;
        }
        push_directory_candidates(candidates, package_path.clone(), executable);
        let Ok(versions) = fs::read_dir(package_path) else {
            continue;
        };
        for version in versions.flatten().take(MAX_VERSIONS_PER_PACKAGE) {
            let version_path = version.path();
            if !version
                .file_type()
                .map(|kind| kind.is_dir())
                .unwrap_or(false)
            {
                continue;
            }
            push_directory_candidates(candidates, version_path.clone(), executable);
            push_directory_candidates(candidates, version_path.join("bin"), executable);
        }
    }
}

#[cfg(windows)]
fn windows_where_candidates(executable: &str) -> Vec<PathBuf> {
    const MAX_OUTPUT_BYTES: usize = 64 * 1024;
    const MAX_RESULTS: usize = 32;
    let mut where_paths = Vec::new();
    if let Some(root) = std::env::var_os("SystemRoot") {
        where_paths.push(PathBuf::from(root).join("System32").join("where.exe"));
    }
    where_paths.push(PathBuf::from(r"C:\Windows\System32\where.exe"));

    let mut candidates = Vec::new();
    for where_path in where_paths {
        if !is_backend_file(&where_path) {
            continue;
        }
        let Ok(output) = Command::new(&where_path)
            .arg(format!("{executable}.exe"))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        else {
            continue;
        };
        if !output.status.success() || output.stdout.len() > MAX_OUTPUT_BYTES {
            continue;
        }
        for line in String::from_utf8_lossy(&output.stdout)
            .lines()
            .take(MAX_RESULTS)
        {
            let line = line.trim();
            let line = line
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(line)
                .trim();
            let candidate = PathBuf::from(line);
            if candidate.is_absolute() {
                push_candidate(&mut candidates, candidate);
            }
        }
        if !candidates.is_empty() {
            break;
        }
    }
    candidates
}

fn backend_candidates(executable: &str, adjacent_directory: Option<&Path>) -> Vec<PathBuf> {
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
        if let Some(candidate) = windows_search_path(executable) {
            push_candidate(&mut candidates, candidate);
        }
    }

    if let Ok(current_exe) = std::env::current_exe()
        && let Some(directory) = current_exe.parent()
    {
        push_directory_candidates(&mut candidates, directory.to_path_buf(), executable);
    }
    if let Ok(current_directory) = std::env::current_dir() {
        push_directory_candidates(&mut candidates, current_directory, executable);
    }
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

    #[cfg(windows)]
    for candidate in windows_where_candidates(executable) {
        push_candidate(&mut candidates, candidate);
    }

    candidates
}

fn is_backend_file(path: &Path) -> bool {
    // O worker recebe apenas caminhos absolutos. `metadata` segue symlinks e
    // junctions, necessários para instalações via gerenciadores de pacotes e
    // links do sistema; continua recusando diretórios.
    path.is_absolute()
        && fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
}

#[cfg(test)]
fn resolve_backend_from_candidates(
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
fn resolve_backend_with_adjacent(
    executable: &'static str,
    adjacent_directory: Option<&Path>,
) -> Result<BackendResolution, ConversionError> {
    let candidates = backend_candidates(executable, adjacent_directory);
    resolve_backend_from_candidates(executable, &candidates)
}

#[cfg(test)]
fn resolve_backend(executable: &'static str) -> Result<BackendResolution, ConversionError> {
    resolve_backend_with_adjacent(executable, None)
}

fn is_backend_retryable_error(error: &ConversionError) -> bool {
    matches!(
        error,
        ConversionError::BackendUnavailable { .. }
            | ConversionError::Process { .. }
            | ConversionError::Timeout { .. }
            | ConversionError::OutputValidationFailed { .. }
    )
}

fn spawn_ffmpeg(
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
    let ffmpeg_candidates = backend_candidates("ffmpeg", None);
    let ffmpeg_paths = ffmpeg_candidates
        .iter()
        .filter(|candidate| is_backend_file(candidate))
        .cloned()
        .collect::<Vec<_>>();
    if ffmpeg_paths.is_empty() {
        return Err(ConversionError::BackendUnavailable {
            executable: "ffmpeg",
            attempts: ffmpeg_candidates.len(),
        });
    }

    let result = (|| {
        let mut last_backend_error = None;
        for ffmpeg in ffmpeg_paths {
            if cancel.load(Ordering::Acquire) {
                return Err(ConversionError::Cancelled);
            }
            let ffprobe_candidates = backend_candidates("ffprobe", ffmpeg.parent());
            let ffprobe_paths = ffprobe_candidates
                .iter()
                .filter(|candidate| is_backend_file(candidate))
                .cloned()
                .collect::<Vec<_>>();
            if ffprobe_paths.is_empty() {
                last_backend_error = Some(ConversionError::BackendUnavailable {
                    executable: "ffprobe",
                    attempts: ffprobe_candidates.len(),
                });
                continue;
            }

            for ffprobe in ffprobe_paths {
                if cancel.load(Ordering::Acquire) {
                    return Err(ConversionError::Cancelled);
                }
                let _ = fs::remove_file(&temporary);
                let attempt = (|| {
                    spawn_ffmpeg(&ffmpeg, &source, &temporary, kind, cancel, &mut stage)?;
                    stage(ConversionStage::Validating);
                    validate_output(&ffprobe, &temporary, kind, cancel)
                })();
                match attempt {
                    Ok(()) => {
                        if cancel.load(Ordering::Acquire) {
                            return Err(ConversionError::Cancelled);
                        }
                        stage(ConversionStage::Publishing);
                        publish_file_no_replace(&temporary, &destination)
                            .map_err(|error| map_destination_error(error, &destination))?;
                        return Ok(ConversionReport {
                            source: source.clone(),
                            destination: destination.clone(),
                            codec: kind.expected_codec(),
                        });
                    }
                    Err(error) if is_backend_retryable_error(&error) => {
                        last_backend_error = Some(error);
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Err(
            last_backend_error.unwrap_or(ConversionError::BackendUnavailable {
                executable: "ffmpeg",
                attempts: ffmpeg_candidates.len(),
            }),
        )
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        ConversionError, ConversionKind, convert_file, output_path,
        push_path_or_directory_candidates, resolve_backend, resolve_backend_from_candidates,
    };
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
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
        let root = std::env::current_dir().unwrap().join("fixtures");
        let jxl = output_path(&root.join("foto.jpg"), ConversionKind::JpegXl).unwrap();
        assert_eq!(jxl, root.join("foto.jxl"));
        let same = output_path(&root.join("foto.jxl"), ConversionKind::JpegXl).unwrap();
        assert_eq!(same, root.join("foto.converted.jxl"));
    }

    #[test]
    fn resolvedor_tenta_candidatos_em_ordem_e_aceita_apenas_arquivo_regular() {
        let root = std::env::temp_dir().join(format!(
            "rovex-backend-resolver-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).expect("criar diretório do teste");
        let missing = root.join("missing");
        let backend = root.join("ffmpeg-real");
        fs::write(&backend, b"backend de teste").expect("criar backend de teste");
        let candidates = vec![missing, backend.clone(), backend.clone()];
        let resolved = resolve_backend_from_candidates("ffmpeg", &candidates)
            .expect("resolver deve avançar até o arquivo regular");
        assert_eq!(resolved.path, backend);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn override_sem_extensao_tambem_considera_executavel_dentro_da_pasta() {
        let directory = std::env::temp_dir().join("rovex-ffmpeg-install");
        let mut candidates = Vec::new();
        push_path_or_directory_candidates(&mut candidates, directory.clone(), "ffmpeg");
        assert!(candidates.contains(&directory));
        assert!(candidates.contains(&directory.with_extension("exe")));
        assert!(candidates.contains(&directory.join("ffmpeg")));
        assert!(candidates.contains(&directory.join("ffmpeg.exe")));
    }

    #[test]
    fn resolvedor_recusa_caminho_relativo() {
        let backend = PathBuf::from("ffmpeg.exe");
        let error = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&backend))
            .expect_err("o worker não deve receber caminho relativo");
        assert!(matches!(
            error,
            ConversionError::BackendUnavailable {
                executable: "ffmpeg",
                attempts: 1
            }
        ));
    }

    #[test]
    fn resolvedor_recusa_diretorio_mesmo_com_nome_de_backend() {
        let root = std::env::temp_dir().join(format!(
            "rovex-backend-resolver-directory-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("criar diretório do teste");
        let directory = root.join("ffmpeg.exe");
        fs::create_dir(&directory).expect("criar diretório com nome de executável");
        let error = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&directory))
            .expect_err("diretório não pode ser tratado como backend");
        assert!(matches!(
            error,
            ConversionError::BackendUnavailable {
                executable: "ffmpeg",
                attempts: 1
            }
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn resolvedor_aceita_link_para_backend_regular() {
        let root = std::env::temp_dir().join(format!(
            "rovex-backend-symlink-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).expect("criar diretório do teste");
        let backend = root.join("ffmpeg-real");
        let link = root.join("ffmpeg-link");
        fs::write(&backend, b"backend de teste").expect("criar backend de teste");
        symlink(&backend, &link).expect("criar link do backend");
        let resolved = resolve_backend_from_candidates("ffmpeg", std::slice::from_ref(&link))
            .expect("resolver deve aceitar link para arquivo regular");
        assert_eq!(resolved.path, link);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolvedor_retorna_erro_estruturado_com_numero_de_tentativas() {
        let candidates = vec![
            std::env::temp_dir().join("rovex-missing-ffmpeg-a"),
            std::env::temp_dir().join("rovex-missing-ffmpeg-b"),
        ];
        let error = resolve_backend_from_candidates("ffmpeg", &candidates)
            .expect_err("nenhum candidato deve ser tratado como backend");
        assert!(matches!(
            error,
            ConversionError::BackendUnavailable {
                executable: "ffmpeg",
                attempts: 2
            }
        ));
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
        let ffmpeg = resolve_backend("ffmpeg").expect("resolver encontrar ffmpeg para fixture");
        let create_image = Command::new(&ffmpeg.path)
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
        let create_audio = Command::new(&ffmpeg.path)
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
