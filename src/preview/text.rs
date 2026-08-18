use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

pub const TEXT_PREVIEW_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewText {
    pub source: PathBuf,
    pub encoding: String,
    pub text: String,
    pub bytes_read: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPreviewError {
    NotRegularFile(PathBuf),
    Binary(PathBuf),
    InvalidUtf8(PathBuf),
    InvalidUtf16(PathBuf),
    Io { path: PathBuf, message: String },
}

impl std::fmt::Display for TextPreviewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegularFile(path) => {
                write!(formatter, "não é um arquivo regular: {}", path.display())
            }
            Self::Binary(path) => write!(
                formatter,
                "conteúdo binário não é exibido como texto: {}",
                path.display()
            ),
            Self::InvalidUtf8(path) => {
                write!(formatter, "texto UTF-8 inválido: {}", path.display())
            }
            Self::InvalidUtf16(path) => {
                write!(formatter, "texto UTF-16 inválido: {}", path.display())
            }
            Self::Io { path, message } => {
                write!(
                    formatter,
                    "falha de leitura de texto em {}: {}",
                    path.display(),
                    message
                )
            }
        }
    }
}

impl std::error::Error for TextPreviewError {}

pub fn decode_text_preview(path: &Path) -> Result<PreviewText, TextPreviewError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| TextPreviewError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    if !metadata.is_file() || is_reparse_point(path) {
        return Err(TextPreviewError::NotRegularFile(path.to_path_buf()));
    }
    let mut file = File::open(path).map_err(|error| TextPreviewError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let mut bytes = Vec::with_capacity(TEXT_PREVIEW_MAX_BYTES + 1);
    file.by_ref()
        .take((TEXT_PREVIEW_MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| TextPreviewError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let truncated = bytes.len() > TEXT_PREVIEW_MAX_BYTES;
    if truncated {
        bytes.truncate(TEXT_PREVIEW_MAX_BYTES);
    }
    let bytes_read = bytes.len();
    let (encoding, text) = decode_bytes(&bytes, path, truncated)?;
    Ok(PreviewText {
        source: path.to_path_buf(),
        encoding: encoding.to_owned(),
        text,
        bytes_read,
        truncated,
    })
}

fn decode_bytes(
    bytes: &[u8],
    path: &Path,
    truncated: bool,
) -> Result<(&'static str, String), TextPreviewError> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return decode_utf8(&bytes[3..], path, truncated).map(|text| ("UTF-8 BOM", text));
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        return decode_utf16(&bytes[2..], path, true, truncated).map(|text| ("UTF-16 LE", text));
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16(&bytes[2..], path, false, truncated).map(|text| ("UTF-16 BE", text));
    }
    if contains_binary_marker(bytes) {
        return Err(TextPreviewError::Binary(path.to_path_buf()));
    }
    decode_utf8(bytes, path, truncated).map(|text| ("UTF-8", text))
}

fn decode_utf8(bytes: &[u8], path: &Path, truncated: bool) -> Result<String, TextPreviewError> {
    match std::str::from_utf8(bytes) {
        Ok(text) => Ok(text.to_owned()),
        Err(error) if truncated && error.error_len().is_none() => {
            let valid = &bytes[..error.valid_up_to()];
            Ok(std::str::from_utf8(valid)
                .map_err(|_| TextPreviewError::InvalidUtf8(path.to_path_buf()))?
                .to_owned())
        }
        Err(_) => Err(TextPreviewError::InvalidUtf8(path.to_path_buf())),
    }
}

fn decode_utf16(
    bytes: &[u8],
    path: &Path,
    little_endian: bool,
    truncated: bool,
) -> Result<String, TextPreviewError> {
    let usable_len = if bytes.len().is_multiple_of(2) || !truncated {
        bytes.len()
    } else {
        bytes.len() - 1
    };
    if usable_len % 2 != 0 {
        return Err(TextPreviewError::InvalidUtf16(path.to_path_buf()));
    }
    let mut units = bytes[..usable_len]
        .chunks_exact(2)
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect::<Vec<_>>();
    match String::from_utf16(&units) {
        Ok(text) => Ok(text),
        Err(_) if truncated => {
            units.pop();
            String::from_utf16(&units)
                .map_err(|_| TextPreviewError::InvalidUtf16(path.to_path_buf()))
        }
        Err(_) => Err(TextPreviewError::InvalidUtf16(path.to_path_buf())),
    }
}

fn contains_binary_marker(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .any(|byte| *byte == 0 || (*byte < 0x20 && !matches!(*byte, b'\n' | b'\r' | b'\t' | 0x0c)))
}

fn is_reparse_point(path: &Path) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return true;
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("rovex-text-preview-{label}-{stamp}.txt"))
    }

    #[test]
    fn decodes_utf8_bom_without_displaying_signature() {
        let path = temp_path("utf8-bom");
        fs::write(&path, [0xef, 0xbb, 0xbf, b'O', b'l', 0xc3, 0xa1]).expect("write");
        let preview = decode_text_preview(&path).expect("text");
        assert_eq!(preview.encoding, "UTF-8 BOM");
        assert_eq!(preview.text, "Olá");
        assert!(!preview.truncated);
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn decodes_utf16_le_bom() {
        let path = temp_path("utf16-le");
        let mut bytes = vec![0xff, 0xfe];
        for unit in "Olá".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        fs::write(&path, bytes).expect("write");
        let preview = decode_text_preview(&path).expect("text");
        assert_eq!(preview.encoding, "UTF-16 LE");
        assert_eq!(preview.text, "Olá");
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn decodes_utf16_be_bom() {
        let path = temp_path("utf16-be");
        let mut bytes = vec![0xfe, 0xff];
        for unit in "Rovex".encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        fs::write(&path, bytes).expect("write");
        let preview = decode_text_preview(&path).expect("text");
        assert_eq!(preview.encoding, "UTF-16 BE");
        assert_eq!(preview.text, "Rovex");
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn rejects_binary_and_invalid_utf8() {
        let binary = temp_path("binary");
        fs::write(&binary, [0x41, 0x00, 0x42]).expect("write");
        assert!(matches!(
            decode_text_preview(&binary),
            Err(TextPreviewError::Binary(_))
        ));
        fs::remove_file(binary).expect("cleanup");

        let invalid = temp_path("invalid");
        fs::write(&invalid, [0xc3, 0x28]).expect("write");
        assert!(matches!(
            decode_text_preview(&invalid),
            Err(TextPreviewError::InvalidUtf8(_))
        ));
        fs::remove_file(invalid).expect("cleanup");
    }

    #[test]
    fn truncates_without_splitting_utf8() {
        let path = temp_path("truncated");
        let mut content = "á".repeat(TEXT_PREVIEW_MAX_BYTES);
        content.push_str(" fim");
        fs::write(&path, content).expect("write");
        let preview = decode_text_preview(&path).expect("text");
        assert!(preview.truncated);
        assert!(preview.text.len() <= TEXT_PREVIEW_MAX_BYTES);
        assert!(std::str::from_utf8(preview.text.as_bytes()).is_ok());
        fs::remove_file(path).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_without_reading_target() {
        use std::os::unix::fs::symlink;
        let target = temp_path("target");
        let link = temp_path("link");
        fs::write(&target, "segredo").expect("write");
        symlink(&target, &link).expect("symlink");
        assert!(matches!(
            decode_text_preview(&link),
            Err(TextPreviewError::NotRegularFile(_))
        ));
        fs::remove_file(link).expect("cleanup link");
        fs::remove_file(target).expect("cleanup target");
    }

    #[test]
    fn empty_file_is_valid_utf8_preview() {
        let path = temp_path("empty");
        File::create(&path).expect("create");
        let preview = decode_text_preview(&path).expect("empty text");
        assert_eq!(preview.text, "");
        assert_eq!(preview.bytes_read, 0);
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn io_error_is_typed() {
        let path = temp_path("missing");
        assert!(matches!(
            decode_text_preview(&path),
            Err(TextPreviewError::Io { .. })
        ));
    }
}
