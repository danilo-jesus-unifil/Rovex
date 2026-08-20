use super::backend::{push_directory_candidates, push_path_or_directory_candidates};
use std::ffi::OsString;
use std::fs;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

#[cfg(windows)]
pub(super) fn windows_wide_null(value: &str) -> Vec<u16> {
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
pub(super) fn windows_registry_access_modes() -> [u32; 3] {
    use windows_sys::Win32::System::Registry::{KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY};
    [
        KEY_READ,
        KEY_READ | KEY_WOW64_64KEY,
        KEY_READ | KEY_WOW64_32KEY,
    ]
}

#[cfg(windows)]
pub(super) fn windows_registry_value(
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
pub(super) fn windows_persistent_path_entries() -> Vec<PathBuf> {
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
pub(super) fn windows_app_path_entries(executable: &str) -> Vec<PathBuf> {
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
pub(super) fn windows_winget_package_candidates(
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
