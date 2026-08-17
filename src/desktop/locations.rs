use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LocationEntry {
    pub(super) label: String,
    pub(super) path: PathBuf,
}

fn add_location(locations: &mut Vec<LocationEntry>, label: &str, path: PathBuf) {
    if !path.is_dir() || locations.iter().any(|location| location.path == path) {
        return;
    }
    locations.push(LocationEntry {
        label: label.to_owned(),
        path,
    });
}

#[cfg(windows)]
fn windows_known_folder_path(folder_id: &windows_sys::core::GUID) -> Option<PathBuf> {
    use windows_sys::Win32::Foundation::{RPC_E_CHANGED_MODE, S_FALSE, S_OK};
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
    };
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;

    // SAFETY: o primeiro parâmetro é reservado e nulo; a flag é uma constante válida.
    let com_result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    let can_use_com = com_result >= 0 || com_result == RPC_E_CHANGED_MODE;
    let mut raw_path = std::ptr::null_mut();
    let path = if can_use_com
        // SAFETY: folder_id aponta para uma constante estática, o token nulo pede o usuário atual,
        // e raw_path permanece válido como ponteiro de saída para a API.
        && unsafe { SHGetKnownFolderPath(folder_id, 0, std::ptr::null_mut(), &mut raw_path) } >= 0
        && !raw_path.is_null()
    {
        let mut length = 0usize;
        // SAFETY: SHGetKnownFolderPath retornou uma string UTF-16 terminada em NUL.
        while unsafe { *raw_path.add(length) } != 0 {
            length += 1;
        }
        // SAFETY: raw_path é não nulo e length foi medido até o terminador NUL.
        let path = OsString::from_wide(unsafe { std::slice::from_raw_parts(raw_path, length) });
        Some(PathBuf::from(path))
    } else {
        None
    };
    if !raw_path.is_null() {
        // SAFETY: a memória foi alocada pela API Shell e ainda não foi liberada.
        unsafe { CoTaskMemFree(raw_path.cast()) };
    }
    if com_result == S_OK || com_result == S_FALSE {
        // SAFETY: só desfazemos uma inicialização COM realizada por esta chamada.
        unsafe { CoUninitialize() };
    }
    path
}

#[cfg(windows)]
fn windows_known_folder_specs() -> &'static [(&'static str, &'static windows_sys::core::GUID)] {
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music,
        FOLDERID_Objects3D, FOLDERID_Pictures, FOLDERID_Videos,
    };
    &[
        ("Área de Trabalho", &FOLDERID_Desktop),
        ("Documentos", &FOLDERID_Documents),
        ("Downloads", &FOLDERID_Downloads),
        ("Imagens", &FOLDERID_Pictures),
        ("Vídeos", &FOLDERID_Videos),
        ("Músicas", &FOLDERID_Music),
        ("Objetos 3D", &FOLDERID_Objects3D),
    ]
}

fn user_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

pub(super) fn default_locations(initial_path: &Path) -> Vec<LocationEntry> {
    let mut locations = Vec::new();
    if let Some(home) = user_home() {
        add_location(&mut locations, "Início", home.clone());
        #[cfg(not(windows))]
        {
            add_location(&mut locations, "Área de Trabalho", home.join("Desktop"));
            add_location(&mut locations, "Documentos", home.join("Documents"));
            add_location(&mut locations, "Downloads", home.join("Downloads"));
            add_location(&mut locations, "Imagens", home.join("Pictures"));
            add_location(&mut locations, "Vídeos", home.join("Videos"));
            add_location(&mut locations, "Músicas", home.join("Music"));
            add_location(&mut locations, "Objetos 3D", home.join("3D Objects"));
        }
    }
    #[cfg(windows)]
    {
        let mut known_folder_count = 0;
        for (label, folder_id) in windows_known_folder_specs() {
            if let Some(path) = windows_known_folder_path(folder_id) {
                known_folder_count += 1;
                add_location(&mut locations, label, path);
            }
        }
        if known_folder_count == 0
            && let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from)
        {
            add_location(&mut locations, "Área de Trabalho", home.join("Desktop"));
            add_location(&mut locations, "Documentos", home.join("Documents"));
            add_location(&mut locations, "Downloads", home.join("Downloads"));
        }
    }
    add_location(&mut locations, "Pasta atual", initial_path.to_path_buf());
    #[cfg(unix)]
    add_location(&mut locations, "Sistema", PathBuf::from("/"));
    locations
}
