use super::{SETTINGS_VERSION, Settings, SettingsError};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub(super) fn serialize_settings(settings: &Settings) -> String {
    let path = settings
        .last_path
        .as_deref()
        .filter(|path| path.is_absolute())
        .map(|path| hex_encode(os_string_bytes(path.as_os_str())))
        .unwrap_or_default();
    format!(
        "# Rovex settings v{SETTINGS_VERSION}\nversion={SETTINGS_VERSION}\nshow_hidden_files={}\nsort_column={}\nsort_ascending={}\nlast_path_hex={path}\n",
        bool_value(settings.show_hidden_files),
        settings.sort_column,
        bool_value(settings.sort_ascending),
    )
}

pub(super) fn parse_settings(path: &Path, text: &str) -> Result<Settings, SettingsError> {
    let mut version = None;
    let mut show_hidden_files = None;
    let mut sort_column = None;
    let mut sort_ascending = None;
    let mut last_path_hex = None;

    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(invalid_line(path, index, "a linha não contém '='"));
        };
        match key {
            "version" => assign(
                &mut version,
                parse_u32(path, index, value, "version")?,
                path,
                index,
            )?,
            "show_hidden_files" => assign(
                &mut show_hidden_files,
                parse_bool(path, index, value)?,
                path,
                index,
            )?,
            "sort_column" => assign(
                &mut sort_column,
                parse_i32(path, index, value, "sort_column")?,
                path,
                index,
            )?,
            "sort_ascending" => assign(
                &mut sort_ascending,
                parse_bool(path, index, value)?,
                path,
                index,
            )?,
            "last_path_hex" => assign(&mut last_path_hex, value.to_owned(), path, index)?,
            _ => {}
        }
    }

    if version != Some(SETTINGS_VERSION) {
        return Err(SettingsError::invalid(
            path,
            format!("versão ausente ou não suportada: {version:?}"),
        ));
    }
    let sort_column = sort_column.ok_or_else(|| missing_key(path, "sort_column"))?;
    if !(0..=5).contains(&sort_column) {
        return Err(SettingsError::invalid(
            path,
            format!("sort_column fora do intervalo: {sort_column}"),
        ));
    }
    let last_path = match last_path_hex.as_deref() {
        None | Some("") => None,
        Some(encoded) => decode_path(path, encoded)?,
    };

    Ok(Settings {
        last_path,
        show_hidden_files: show_hidden_files
            .ok_or_else(|| missing_key(path, "show_hidden_files"))?,
        sort_column,
        sort_ascending: sort_ascending.ok_or_else(|| missing_key(path, "sort_ascending"))?,
    })
}

fn assign<T>(
    slot: &mut Option<T>,
    value: T,
    path: &Path,
    line: usize,
) -> Result<(), SettingsError> {
    if slot.replace(value).is_some() {
        return Err(invalid_line(path, line, "chave duplicada"));
    }
    Ok(())
}

fn bool_value(value: bool) -> u8 {
    u8::from(value)
}

fn parse_bool(path: &Path, line: usize, value: &str) -> Result<bool, SettingsError> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(invalid_line(path, line, "booleano deve ser 0 ou 1")),
    }
}

fn parse_u32(path: &Path, line: usize, value: &str, key: &str) -> Result<u32, SettingsError> {
    value
        .parse()
        .map_err(|_| invalid_line(path, line, &format!("{key} não é um inteiro válido")))
}

fn parse_i32(path: &Path, line: usize, value: &str, key: &str) -> Result<i32, SettingsError> {
    value
        .parse()
        .map_err(|_| invalid_line(path, line, &format!("{key} não é um inteiro válido")))
}

fn invalid_line(path: &Path, line: usize, reason: &str) -> SettingsError {
    SettingsError::invalid(path, format!("linha {}: {reason}", line + 1))
}

fn missing_key(path: &Path, key: &str) -> SettingsError {
    SettingsError::invalid(path, format!("chave ausente: {key}"))
}

fn decode_path(path: &Path, encoded: &str) -> Result<Option<PathBuf>, SettingsError> {
    let bytes = hex_decode(encoded)
        .ok_or_else(|| SettingsError::invalid(path, "last_path_hex não é hexadecimal válido"))?;
    let os_string = os_string_from_bytes(bytes).ok_or_else(|| {
        SettingsError::invalid(path, "last_path_hex não representa um caminho válido")
    })?;
    let decoded = PathBuf::from(os_string);
    if !decoded.is_absolute() {
        return Err(SettingsError::invalid(
            path,
            "last_path_hex representa um caminho relativo",
        ));
    }
    Ok(Some(decoded))
}

fn hex_encode(bytes: Vec<u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Some((hex_digit(pair[0])? << 4) | hex_digit(pair[1])?))
        .collect()
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(unix)]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}

#[cfg(windows)]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().flat_map(u16::to_le_bytes).collect()
}

#[cfg(not(any(unix, windows)))]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().into_owned().into_bytes()
}

#[cfg(unix)]
fn os_string_from_bytes(bytes: Vec<u8>) -> Option<OsString> {
    use std::os::unix::ffi::OsStringExt;
    Some(OsString::from_vec(bytes))
}

#[cfg(windows)]
fn os_string_from_bytes(bytes: Vec<u8>) -> Option<OsString> {
    use std::os::windows::ffi::OsStringExt;
    let units = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    bytes
        .len()
        .is_multiple_of(2)
        .then(|| OsString::from_wide(&units))
}

#[cfg(not(any(unix, windows)))]
fn os_string_from_bytes(bytes: Vec<u8>) -> Option<OsString> {
    String::from_utf8(bytes).ok().map(OsString::from)
}
