use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum OpenWithError {
    InvalidTarget(PathBuf),
    Unsupported,
    ComInitialization(i32),
    DialogFailed(i32),
}

impl fmt::Display for OpenWithError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(path) => write!(
                formatter,
                "Open With exige um arquivo regular sem redirecionamento especial: {}",
                path.display()
            ),
            Self::Unsupported => {
                write!(formatter, "Open With não está disponível nesta plataforma")
            }
            Self::ComInitialization(hr) => {
                write!(
                    formatter,
                    "não foi possível inicializar o Shell do Windows (HRESULT 0x{hr:08X})"
                )
            }
            Self::DialogFailed(hr) => {
                write!(formatter, "o diálogo Open With falhou (HRESULT 0x{hr:08X})")
            }
        }
    }
}

impl std::error::Error for OpenWithError {}

pub fn is_supported() -> bool {
    cfg!(windows)
}

pub fn validate_file(path: &Path) -> Result<PathBuf, OpenWithError> {
    if !path.is_absolute() || !path.is_file() || is_reparse_point(path) {
        return Err(OpenWithError::InvalidTarget(path.to_path_buf()));
    }
    Ok(path.to_path_buf())
}

pub fn open_with_file(path: &Path) -> Result<(), OpenWithError> {
    let path = validate_file(path)?;
    #[cfg(windows)]
    {
        open_windows_dialog(&path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(OpenWithError::Unsupported)
    }
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

#[cfg(windows)]
const OAIF_EXEC: i32 = 0x4;

#[cfg(windows)]
#[repr(C)]
struct OpenAsInfo {
    file: *const u16,
    class: *const u16,
    flags: i32,
}

#[cfg(windows)]
#[link(name = "shell32")]
unsafe extern "system" {
    #[link_name = "SHOpenWithDialog"]
    fn sh_open_with_dialog(parent: *mut std::ffi::c_void, info: *const OpenAsInfo) -> i32;
}

#[cfg(windows)]
fn open_windows_dialog(path: &Path) -> Result<(), OpenWithError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize,
    };

    let com_result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if com_result == RPC_E_CHANGED_MODE {
        return Err(OpenWithError::ComInitialization(com_result));
    }
    if com_result < 0 {
        return Err(OpenWithError::ComInitialization(com_result));
    }

    let wide = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let info = OpenAsInfo {
        file: wide.as_ptr(),
        class: std::ptr::null(),
        flags: OAIF_EXEC,
    };
    let result = unsafe { sh_open_with_dialog(std::ptr::null_mut(), &info) };
    if com_result == S_OK || com_result == S_FALSE {
        unsafe { CoUninitialize() };
    }
    if result >= 0 {
        Ok(())
    } else {
        Err(OpenWithError::DialogFailed(result))
    }
}

#[cfg(test)]
mod tests {
    use super::{OpenWithError, validate_file};
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_file() -> (std::path::PathBuf, std::path::PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("rovex-open-with-test-{id}"));
        fs::create_dir_all(&root).expect("o diretório de teste deve ser criado");
        let file = root.join("relatório com espaço.txt");
        fs::write(&file, b"texto").expect("o arquivo de teste deve ser criado");
        (root, file)
    }

    #[test]
    fn accepts_absolute_regular_file_without_canonicalizing_name() {
        let (root, file) = temporary_file();
        assert_eq!(validate_file(&file).unwrap(), file);
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[test]
    fn rejects_directory_relative_and_missing_targets() {
        let (root, file) = temporary_file();
        assert!(matches!(
            validate_file(&root),
            Err(OpenWithError::InvalidTarget(_))
        ));
        assert!(matches!(
            validate_file(Path::new("relativo.txt")),
            Err(OpenWithError::InvalidTarget(_))
        ));
        assert!(matches!(
            validate_file(&file.with_file_name("ausente.txt")),
            Err(OpenWithError::InvalidTarget(_))
        ));
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_without_following_target() {
        let (root, file) = temporary_file();
        let link = root.join("link.txt");
        std::os::unix::fs::symlink(&file, &link).expect("o symlink deve ser criado");
        assert!(matches!(
            validate_file(&link),
            Err(OpenWithError::InvalidTarget(_))
        ));
        fs::remove_file(link).expect("o symlink deve ser removido");
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[cfg(windows)]
    #[test]
    fn openasinfo_uses_explicit_execute_flag_only() {
        assert_eq!(super::OAIF_EXEC, 0x4_i32);
    }
}
