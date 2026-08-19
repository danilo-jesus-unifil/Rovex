use super::error::{OperationError, from_io};
use std::io;
use std::path::Path;

use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::UI::Shell::{
    FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_NORECURSION, FOF_SILENT,
    SHFILEOPSTRUCTW, SHFileOperationW,
};

const RECYCLE_FLAGS: u16 =
    (FOF_ALLOWUNDO | FOF_NOERRORUI | FOF_NOCONFIRMATION | FOF_NORECURSION | FOF_SILENT) as u16;

pub(super) fn delete_to_recycle_bin(path: &Path) -> Result<(), OperationError> {
    let mut source = path.as_os_str().encode_wide().collect::<Vec<_>>();
    source.extend([0, 0]);
    let mut operation = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: source.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: RECYCLE_FLAGS,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };

    // SAFETY: `source` permanece vivo e imutável durante a chamada; o buffer é
    // uma lista de um caminho terminada por dois NULs, como exige a Shell API.
    let result = unsafe { SHFileOperationW(&mut operation) };
    if result != 0 {
        return Err(from_io(
            "mover para a Lixeira",
            path,
            io::Error::from_raw_os_error(result),
        ));
    }
    if operation.fAnyOperationsAborted != 0 {
        return Err(from_io(
            "mover para a Lixeira",
            path,
            io::Error::new(io::ErrorKind::Interrupted, "operação cancelada pelo Shell"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::OsStrExt;
    use std::ffi::OsStr;

    #[test]
    fn path_buffer_is_double_null_terminated() {
        let mut buffer = OsStr::new("C:\\Rovex\\arquivo.txt")
            .encode_wide()
            .collect::<Vec<_>>();
        buffer.extend([0, 0]);
        assert_eq!(buffer[buffer.len() - 2..], [0, 0]);
        assert!(!buffer[..buffer.len() - 2].contains(&0));
    }
}
