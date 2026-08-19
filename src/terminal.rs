use std::fmt;
#[cfg(windows)]
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum TerminalError {
    InvalidTarget(PathBuf),
    Unsupported,
    Unavailable { attempts: Vec<String> },
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(path) => write!(
                formatter,
                "não foi possível abrir um terminal: o diretório não é válido: {}",
                path.display()
            ),
            Self::Unsupported => write!(
                formatter,
                "abrir terminal não está disponível nesta plataforma"
            ),
            Self::Unavailable { attempts } => write!(
                formatter,
                "nenhum terminal disponível foi iniciado ({})",
                attempts.join("; ")
            ),
        }
    }
}

impl std::error::Error for TerminalError {}

pub fn is_supported() -> bool {
    cfg!(windows)
}

pub fn target_directory(path: &Path, is_directory: bool) -> Result<PathBuf, TerminalError> {
    let candidate = if is_directory {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| TerminalError::InvalidTarget(path.to_path_buf()))?
    };
    if !candidate.is_absolute() || !candidate.is_dir() {
        return Err(TerminalError::InvalidTarget(candidate));
    }
    Ok(candidate)
}

pub fn open_terminal_for_item(
    path: &Path,
    is_directory: bool,
) -> Result<&'static str, TerminalError> {
    let directory = target_directory(path, is_directory)?;
    open_terminal_here(&directory)
}

pub fn open_terminal_here(path: &Path) -> Result<&'static str, TerminalError> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(TerminalError::InvalidTarget(path.to_path_buf()));
    }

    #[cfg(windows)]
    {
        open_windows_terminal(path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err(TerminalError::Unsupported)
    }
}

#[cfg(windows)]
struct TerminalCandidate {
    label: &'static str,
    program: &'static str,
    args: Vec<std::ffi::OsString>,
}

#[cfg(windows)]
fn windows_candidates(path: &Path) -> [TerminalCandidate; 3] {
    use std::ffi::OsString;

    [
        TerminalCandidate {
            label: "Windows Terminal",
            program: "wt.exe",
            args: vec![
                OsString::from("-w"),
                OsString::from("new"),
                OsString::from("new-tab"),
                OsString::from("--startingDirectory"),
                path.as_os_str().to_os_string(),
            ],
        },
        TerminalCandidate {
            label: "PowerShell",
            program: "powershell.exe",
            args: vec![OsString::from("-NoLogo"), OsString::from("-NoExit")],
        },
        TerminalCandidate {
            label: "Prompt de Comando",
            program: "cmd.exe",
            args: vec![OsString::from("/D"), OsString::from("/K")],
        },
    ]
}

#[cfg(windows)]
fn open_windows_terminal(path: &Path) -> Result<&'static str, TerminalError> {
    use std::process::Command;

    let mut attempts = Vec::new();
    for candidate in windows_candidates(path) {
        let result = Command::new(candidate.program)
            .args(&candidate.args)
            .current_dir(path)
            .spawn();
        match result {
            Ok(child) => {
                drop(child);
                return Ok(candidate.label);
            }
            Err(error) => attempts.push(format!(
                "{}: {}",
                candidate.program,
                describe_spawn_error(error)
            )),
        }
    }
    Err(TerminalError::Unavailable { attempts })
}

#[cfg(windows)]
fn describe_spawn_error(error: io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "executável não encontrado".to_owned(),
        io::ErrorKind::PermissionDenied => "acesso negado".to_owned(),
        _ => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{TerminalError, target_directory};
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory() -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("rovex-terminal-test-{id}"));
        fs::create_dir_all(&path).expect("o diretório de teste deve ser criado");
        path
    }

    #[test]
    fn file_target_uses_its_existing_parent_directory() {
        let root = temporary_directory();
        let file = root.join("relatório com espaço.txt");
        fs::write(&file, b"x").expect("o arquivo de teste deve ser criado");
        assert_eq!(target_directory(&file, false).unwrap(), root);
        fs::remove_dir_all(root).expect("o diretório de teste deve ser removido");
    }

    #[test]
    fn directory_target_is_preserved_without_canonicalizing_unicode() {
        let root = temporary_directory().join("pasta — segura");
        fs::create_dir_all(&root).expect("a pasta de teste deve ser criada");
        assert_eq!(target_directory(&root, true).unwrap(), root);
        fs::remove_dir_all(root.parent().unwrap()).expect("o diretório de teste deve ser removido");
    }

    #[test]
    fn relative_or_missing_target_is_rejected() {
        assert!(matches!(
            target_directory(Path::new("relativo"), true),
            Err(TerminalError::InvalidTarget(_))
        ));
        assert!(matches!(
            target_directory(Path::new("C:\\Rovex\\ausente"), true),
            Err(TerminalError::InvalidTarget(_))
        ));
    }

    #[cfg(windows)]
    #[test]
    fn windows_terminal_candidate_keeps_path_as_a_separate_argument() {
        let path = Path::new(r"C:\Rovex\pasta com espaço");
        let candidates = super::windows_candidates(path);
        let candidate = &candidates[0].args;
        assert_eq!(candidate[0], "-w");
        assert_eq!(candidate[1], "new");
        assert_eq!(candidate[2], "new-tab");
        assert_eq!(candidate[3], "--startingDirectory");
        assert_eq!(candidate[4], path.as_os_str());
    }
}
