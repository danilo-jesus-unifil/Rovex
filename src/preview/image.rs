use image::{DynamicImage, ImageDecoder, ImageReader, Limits};
use std::fs;
use std::path::{Path, PathBuf};

pub const PREVIEW_MAX_SIDE: u32 = 256;
pub const PREVIEW_MAX_DIMENSION: u32 = 8_192;
pub const PREVIEW_MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
pub const PREVIEW_MAX_DECODE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewLimits {
    pub max_side: u32,
    pub max_dimension: u32,
    pub max_source_bytes: u64,
    pub max_decode_bytes: u64,
}

impl Default for PreviewLimits {
    fn default() -> Self {
        Self {
            max_side: PREVIEW_MAX_SIDE,
            max_dimension: PREVIEW_MAX_DIMENSION,
            max_source_bytes: PREVIEW_MAX_SOURCE_BYTES,
            max_decode_bytes: PREVIEW_MAX_DECODE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewImage {
    pub source: PathBuf,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    NotRegularFile(PathBuf),
    SourceTooLarge {
        path: PathBuf,
        bytes: u64,
    },
    DimensionsTooLarge {
        path: PathBuf,
        width: u32,
        height: u32,
    },
    DecodeTooLarge {
        path: PathBuf,
        bytes: u64,
    },
    Io {
        path: PathBuf,
        message: String,
    },
    Decode {
        path: PathBuf,
        message: String,
    },
}

impl std::fmt::Display for PreviewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegularFile(path) => {
                write!(formatter, "não é um arquivo regular: {}", path.display())
            }
            Self::SourceTooLarge { path, bytes } => write!(
                formatter,
                "arquivo de preview excede o limite de entrada ({} bytes): {}",
                bytes,
                path.display()
            ),
            Self::DimensionsTooLarge {
                path,
                width,
                height,
            } => write!(
                formatter,
                "dimensões de preview excedem o limite ({}x{}): {}",
                width,
                height,
                path.display()
            ),
            Self::DecodeTooLarge { path, bytes } => write!(
                formatter,
                "imagem decodificada excede o limite ({} bytes): {}",
                bytes,
                path.display()
            ),
            Self::Io { path, message } => {
                write!(
                    formatter,
                    "falha de leitura de preview em {}: {}",
                    path.display(),
                    message
                )
            }
            Self::Decode { path, message } => write!(
                formatter,
                "formato de imagem não suportado ou inválido em {}: {}",
                path.display(),
                message
            ),
        }
    }
}

impl std::error::Error for PreviewError {}

fn format_name(format: image::ImageFormat) -> String {
    format!("{format:?}")
}

pub fn decode_thumbnail(path: &Path, limits: PreviewLimits) -> Result<PreviewImage, PreviewError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| PreviewError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    if !metadata.is_file() || is_reparse_point(path) {
        return Err(PreviewError::NotRegularFile(path.to_path_buf()));
    }
    if metadata.len() > limits.max_source_bytes {
        return Err(PreviewError::SourceTooLarge {
            path: path.to_path_buf(),
            bytes: metadata.len(),
        });
    }

    let dimensions_reader = ImageReader::open(path)
        .map_err(|error| PreviewError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?
        .with_guessed_format()
        .map_err(|error| PreviewError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let (width, height) =
        dimensions_reader
            .into_dimensions()
            .map_err(|error| PreviewError::Decode {
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
    if width > limits.max_dimension || height > limits.max_dimension {
        return Err(PreviewError::DimensionsTooLarge {
            path: path.to_path_buf(),
            width,
            height,
        });
    }
    let reader = ImageReader::open(path)
        .map_err(|error| PreviewError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?
        .with_guessed_format()
        .map_err(|error| PreviewError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let format = reader
        .format()
        .map(format_name)
        .unwrap_or_else(|| "desconhecido".to_owned());
    let mut decoder_limits = Limits::default();
    decoder_limits.max_image_width = Some(limits.max_dimension);
    decoder_limits.max_image_height = Some(limits.max_dimension);
    decoder_limits.max_alloc = Some(limits.max_decode_bytes);
    let mut reader = reader;
    reader.limits(decoder_limits);
    let decoder = reader
        .into_decoder()
        .map_err(|error| PreviewError::Decode {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    if decoder.total_bytes() > limits.max_decode_bytes {
        return Err(PreviewError::DecodeTooLarge {
            path: path.to_path_buf(),
            bytes: decoder.total_bytes(),
        });
    }
    let image = DynamicImage::from_decoder(decoder).map_err(|error| PreviewError::Decode {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let thumbnail = image
        .thumbnail(limits.max_side, limits.max_side)
        .into_rgba8();
    Ok(PreviewImage {
        source: path.to_path_buf(),
        format,
        width: thumbnail.width(),
        height: thumbnail.height(),
        rgba: thumbnail.into_raw(),
    })
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
    use image::{ImageBuffer, ImageFormat, Rgba};
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str, extension: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("rovex-preview-{label}-{stamp}.{extension}"))
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        let image = ImageBuffer::from_pixel(width, height, Rgba([30_u8, 120_u8, 220_u8, 255_u8]));
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("encode png");
        fs::write(path, bytes.into_inner()).expect("write png");
    }

    #[test]
    fn decodes_by_content_and_scales_within_bound() {
        let path = temp_path("valid", "not-an-image");
        write_png(&path, 512, 256);
        let preview = decode_thumbnail(
            &path,
            PreviewLimits {
                max_side: 64,
                ..PreviewLimits::default()
            },
        )
        .expect("decode thumbnail");
        assert_eq!(preview.width, 64);
        assert_eq!(preview.height, 32);
        assert_eq!(preview.rgba.len(), 64 * 32 * 4);
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn rejects_dimensions_before_materializing_image() {
        let path = temp_path("dimensions", "png");
        write_png(&path, 4, 4);
        let error = decode_thumbnail(
            &path,
            PreviewLimits {
                max_dimension: 2,
                ..PreviewLimits::default()
            },
        )
        .expect_err("dimension limit");
        assert!(matches!(error, PreviewError::DimensionsTooLarge { .. }));
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn malformed_content_returns_fallback_error() {
        let path = temp_path("malformed", "jpg");
        fs::write(&path, b"not an image").expect("write malformed");
        let error = decode_thumbnail(&path, PreviewLimits::default()).expect_err("malformed");
        assert!(matches!(error, PreviewError::Decode { .. }));
        fs::remove_file(path).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_is_not_decoded() {
        use std::os::unix::fs::symlink;
        let target = temp_path("target", "png");
        let link = temp_path("link", "png");
        write_png(&target, 2, 2);
        symlink(&target, &link).expect("symlink");
        let error = decode_thumbnail(&link, PreviewLimits::default()).expect_err("symlink");
        assert!(matches!(error, PreviewError::NotRegularFile(_)));
        fs::remove_file(link).expect("cleanup link");
        fs::remove_file(target).expect("cleanup target");
    }
}
