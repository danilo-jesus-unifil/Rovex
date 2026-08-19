mod atomic;
mod format;

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

const SETTINGS_DIRECTORY: &str = "Rovex";
const SETTINGS_FILE_NAME: &str = "settings.v1.conf";
const SETTINGS_VERSION: u32 = 1;
const MAX_SETTINGS_BYTES: u64 = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub last_path: Option<PathBuf>,
    pub show_hidden_files: bool,
    pub sort_column: i32,
    pub sort_ascending: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_path: None,
            show_hidden_files: false,
            sort_column: 1,
            sort_ascending: true,
        }
    }
}

#[derive(Debug)]
pub enum SettingsError {
    Io { path: PathBuf, kind: io::ErrorKind },
    InvalidFormat { path: PathBuf, reason: String },
}

impl SettingsError {
    fn io(path: &Path, error: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            kind: error.kind(),
        }
    }

    pub(super) fn invalid(path: &Path, reason: impl Into<String>) -> Self {
        Self::InvalidFormat {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }

    fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::Io {
                kind: io::ErrorKind::NotFound,
                ..
            }
        )
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, kind } => {
                write!(
                    formatter,
                    "não foi possível acessar {}: {kind}",
                    path.display()
                )
            }
            Self::InvalidFormat { path, reason } => {
                write!(
                    formatter,
                    "configuração inválida em {}: {reason}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for SettingsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn discover() -> Option<Self> {
        default_settings_path().map(Self::from_path)
    }

    pub fn from_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Settings, SettingsError> {
        let metadata = fs::symlink_metadata(&self.path)
            .map_err(|error| SettingsError::io(&self.path, error))?;
        if !metadata.file_type().is_file() {
            return Err(SettingsError::invalid(
                &self.path,
                "o alvo não é um arquivo regular",
            ));
        }
        if metadata.len() > MAX_SETTINGS_BYTES {
            return Err(SettingsError::invalid(
                &self.path,
                format!("o arquivo excede o limite de {MAX_SETTINGS_BYTES} bytes"),
            ));
        }

        let mut file =
            File::open(&self.path).map_err(|error| SettingsError::io(&self.path, error))?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_SETTINGS_BYTES.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| SettingsError::io(&self.path, error))?;
        if bytes.len() as u64 > MAX_SETTINGS_BYTES {
            return Err(SettingsError::invalid(
                &self.path,
                format!("o arquivo excede o limite de {MAX_SETTINGS_BYTES} bytes"),
            ));
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| SettingsError::invalid(&self.path, "o conteúdo não é UTF-8"))?;
        format::parse_settings(&self.path, text)
    }

    pub fn load_or_default(&self) -> Settings {
        match self.load() {
            Ok(settings) => settings,
            Err(error) if error.is_not_found() => Settings::default(),
            Err(error) => {
                eprintln!("aviso: {error}; usando configurações padrão");
                Settings::default()
            }
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        let parent = self.path.parent().ok_or_else(|| {
            SettingsError::invalid(&self.path, "o caminho não possui diretório pai")
        })?;
        fs::create_dir_all(parent).map_err(|error| SettingsError::io(parent, error))?;
        let content = format::serialize_settings(settings);
        atomic::write_and_replace(parent, &self.path, &content)
            .map_err(|error| SettingsError::io(&self.path, error))
    }
}

pub fn default_settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("ROVEX_CONFIG_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(platform_config_directory)?;
    Some(base.join(SETTINGS_DIRECTORY).join(SETTINGS_FILE_NAME))
}

fn platform_config_directory() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .map(|path| path.join("AppData").join("Local"))
            })
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .map(|path| path.join(".config"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{Settings, SettingsStore, default_settings_path};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rovex-settings-{name}-{}-{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn unicode_absolute_path() -> PathBuf {
        #[cfg(windows)]
        {
            PathBuf::from(r"C:\Rovex\ação segura")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/tmp/ação segura")
        }
    }

    #[test]
    fn round_trips_preferences_and_unicode_path() {
        let path = test_path("roundtrip").join("settings.v1.conf");
        let store = SettingsStore::from_path(path.clone());
        let settings = Settings {
            last_path: Some(unicode_absolute_path()),
            show_hidden_files: true,
            sort_column: 4,
            sort_ascending: false,
        };
        store.save(&settings).expect("save");
        assert_eq!(store.load().expect("load"), settings);
        fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn replaces_existing_file_without_leaving_temporary_files() {
        let directory = test_path("replace");
        let path = directory.join("settings.v1.conf");
        let store = SettingsStore::from_path(path.clone());
        store.save(&Settings::default()).expect("first save");
        let updated = Settings {
            last_path: None,
            show_hidden_files: true,
            sort_column: 2,
            sort_ascending: false,
        };
        store.save(&updated).expect("second save");
        assert_eq!(store.load().expect("load"), updated);
        let entries = fs::read_dir(&directory)
            .expect("read directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name(), "settings.v1.conf");
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn missing_or_corrupt_file_falls_back_to_default() {
        let path = test_path("fallback").join("settings.v1.conf");
        let store = SettingsStore::from_path(path.clone());
        assert_eq!(store.load_or_default(), Settings::default());
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(&path, "version=1\nsort_column=99\n").expect("write");
        assert_eq!(store.load_or_default(), Settings::default());
        fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn concurrent_saves_leave_a_valid_complete_file() {
        let directory = test_path("concurrent");
        let path = directory.join("settings.v1.conf");
        let mut workers = Vec::new();
        for index in 0..8 {
            let store = SettingsStore::from_path(path.clone());
            workers.push(std::thread::spawn(move || {
                store
                    .save(&Settings {
                        last_path: None,
                        show_hidden_files: index % 2 == 0,
                        sort_column: index % 6,
                        sort_ascending: index % 3 == 0,
                    })
                    .expect("concurrent save");
            }));
        }
        for worker in workers {
            worker.join().expect("worker");
        }
        let loaded = SettingsStore::from_path(path.clone()).load().expect("load");
        assert!((0..=5).contains(&loaded.sort_column));
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn rejects_directory_as_configuration_file() {
        let directory = test_path("directory-target");
        fs::create_dir_all(&directory).expect("mkdir");
        let store = SettingsStore::from_path(directory.clone());
        assert!(store.load().is_err());
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn accepts_unknown_keys_for_forward_compatibility() {
        let path = test_path("future-key").join("settings.v1.conf");
        let store = SettingsStore::from_path(path.clone());
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(
            &path,
            "version=1\nshow_hidden_files=1\nsort_column=1\nsort_ascending=1\nlast_path_hex=\nfuture_setting=ignored\n",
        )
        .expect("write");
        assert_eq!(
            store.load().expect("load"),
            Settings {
                last_path: None,
                show_hidden_files: true,
                sort_column: 1,
                sort_ascending: true,
            }
        );
        fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn rejects_oversized_configuration() {
        let path = test_path("large").join("settings.v1.conf");
        let store = SettingsStore::from_path(path.clone());
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(&path, vec![b'x'; 16 * 1024 + 1]).expect("write");
        assert!(store.load().is_err());
        fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn default_path_is_absolute_when_environment_is_available() {
        if let Some(path) = default_settings_path() {
            assert!(path.is_absolute());
            assert!(path.ends_with("settings.v1.conf"));
        }
    }
}
