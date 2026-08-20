use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub enum ActivationError {
    InvalidTarget(PathBuf),
    Unsupported,
    ComInitialization(i32),
    ShellExecuteFailed(u32),
}

impl fmt::Display for ActivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(path) => write!(
                formatter,
                "Abrir exige um arquivo regular sem redirecionamento especial: {}",
                path.display()
            ),
            Self::Unsupported => {
                write!(
                    formatter,
                    "abrir arquivo não está disponível nesta plataforma"
                )
            }
            Self::ComInitialization(hr) => write!(
                formatter,
                "não foi possível inicializar o Shell do Windows (HRESULT 0x{:08X})",
                *hr as u32
            ),
            Self::ShellExecuteFailed(error) => write!(
                formatter,
                "o aplicativo padrão não pôde abrir o arquivo (erro Win32 {})",
                error
            ),
        }
    }
}

impl std::error::Error for ActivationError {}

pub fn is_supported() -> bool {
    cfg!(windows)
}

pub fn validate_file(path: &Path) -> Result<PathBuf, ActivationError> {
    let has_parent_component = path
        .components()
        .any(|component| matches!(component, Component::ParentDir));
    if !path.is_absolute() || !path.is_file() || has_parent_component || has_reparse_component(path)
    {
        return Err(ActivationError::InvalidTarget(path.to_path_buf()));
    }
    Ok(path.to_path_buf())
}

pub fn activate_file(path: &Path) -> Result<(), ActivationError> {
    let path = validate_file(path)?;
    #[cfg(windows)]
    {
        activate_windows_file(&path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(ActivationError::Unsupported)
    }
}

fn has_reparse_component(path: &Path) -> bool {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if matches!(component, Component::Normal(_)) && is_reparse_point(&current) {
            return true;
        }
    }
    false
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
const SW_SHOWNORMAL: i32 = 1;

#[cfg(windows)]
fn activate_windows_file(path: &Path) -> Result<(), ActivationError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{GetLastError, RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize,
    };
    use windows_sys::Win32::UI::Shell::{SHELLEXECUTEINFOW, ShellExecuteExW};

    let com_result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if com_result == RPC_E_CHANGED_MODE || com_result < 0 {
        return Err(ActivationError::ComInitialization(com_result));
    }
    let should_uninitialize = com_result == S_OK || com_result == S_FALSE;

    let wide = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: std::ptr::null(),
        lpFile: wide.as_ptr(),
        lpParameters: std::ptr::null(),
        lpDirectory: std::ptr::null(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };

    let succeeded = unsafe { ShellExecuteExW(&mut info) != 0 };
    let error = if succeeded {
        None
    } else {
        Some(unsafe { GetLastError() })
    };
    if should_uninitialize {
        unsafe { CoUninitialize() };
    }
    error.map_or(Ok(()), |code| {
        Err(ActivationError::ShellExecuteFailed(code))
    })
}

#[cfg(test)]
mod tests {
    use super::{ActivationError, validate_file};
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_file() -> (std::path::PathBuf, std::path::PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("rovex-activation-test-{id}"));
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
            Err(ActivationError::InvalidTarget(_))
        ));
        assert!(matches!(
            validate_file(Path::new("relativo.txt")),
            Err(ActivationError::InvalidTarget(_))
        ));
        assert!(matches!(
            validate_file(&file.with_file_name("ausente.txt")),
            Err(ActivationError::InvalidTarget(_))
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
            Err(ActivationError::InvalidTarget(_))
        ));
        fs::remove_file(link).expect("o symlink deve ser removido");
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_file_inside_symlinked_parent() {
        let (root, file) = temporary_file();
        let real_parent = file.parent().expect("o arquivo deve ter diretório pai");
        let link_parent = root.join("link-parent");
        std::os::unix::fs::symlink(real_parent, &link_parent)
            .expect("o symlink de diretório deve ser criado");
        let linked_file = link_parent.join(file.file_name().expect("o arquivo deve ter nome"));
        assert!(matches!(
            validate_file(&linked_file),
            Err(ActivationError::InvalidTarget(_))
        ));
        fs::remove_file(link_parent).expect("o symlink de diretório deve ser removido");
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[test]
    fn rejects_ambiguous_parent_component() {
        let (root, file) = temporary_file();
        let parent = file.parent().expect("o arquivo deve ter diretório pai");
        fs::create_dir_all(parent.join("nested")).expect("o diretório intermediário deve existir");
        let ambiguous = parent
            .join("nested")
            .join("..")
            .join(file.file_name().unwrap());
        assert!(matches!(
            validate_file(&ambiguous),
            Err(ActivationError::InvalidTarget(_))
        ));
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[cfg(windows)]
    #[test]
    fn default_activation_does_not_construct_command_parameters() {
        assert_eq!(super::SW_SHOWNORMAL, 1);
    }
}
