use std::ffi::c_void;
use std::io;
use std::path::Path;

use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{E_POINTER, RPC_E_CHANGED_MODE, S_FALSE, S_OK};
use windows_sys::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows_sys::Win32::UI::Shell::{
    FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOCONFIRMMKDIR, FOF_NOERRORUI,
    FOF_NORECURSION, FOF_SILENT, FOFX_EARLYFAILURE, FOFX_RECYCLEONDELETE,
    SHCreateItemFromParsingName, SHFILEOPSTRUCTW, SHFileOperationW,
};
use windows_sys::core::{GUID, HRESULT, PCWSTR};

use super::error::{OperationError, from_io};

const IFILE_OPERATION_FLAGS: u32 = FOF_NOCONFIRMATION
    | FOF_NOCONFIRMMKDIR
    | FOF_NOERRORUI
    | FOF_NORECURSION
    | FOF_SILENT
    | FOFX_EARLYFAILURE
    | FOFX_RECYCLEONDELETE;

#[repr(C)]
struct IUnknownVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

#[repr(C)]
struct IFileOperationVtbl {
    query_interface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    advise: unsafe extern "system" fn(),
    unadvise: unsafe extern "system" fn(),
    set_operation_flags: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
    set_progress_message: unsafe extern "system" fn(),
    set_progress_dialog: unsafe extern "system" fn(),
    set_properties: unsafe extern "system" fn(),
    set_owner_window: unsafe extern "system" fn(),
    apply_properties_to_item: unsafe extern "system" fn(),
    apply_properties_to_items: unsafe extern "system" fn(),
    rename_item: unsafe extern "system" fn(),
    rename_items: unsafe extern "system" fn(),
    move_item: unsafe extern "system" fn(),
    move_items: unsafe extern "system" fn(),
    copy_item: unsafe extern "system" fn(),
    copy_items: unsafe extern "system" fn(),
    delete_item: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> HRESULT,
    delete_items: unsafe extern "system" fn(),
    new_item: unsafe extern "system" fn(),
    perform_operations: unsafe extern "system" fn(*mut c_void) -> HRESULT,
    get_any_operations_aborted: unsafe extern "system" fn(*mut c_void, *mut i32) -> HRESULT,
}

#[repr(C)]
struct IFileOperation {
    vtable: *const IFileOperationVtbl,
}

struct ComPtr {
    raw: *mut c_void,
}

impl ComPtr {
    fn from_raw(raw: *mut c_void) -> Result<Self, HRESULT> {
        if raw.is_null() {
            Err(E_POINTER)
        } else {
            Ok(Self { raw })
        }
    }

    fn as_file_operation(&self) -> *mut IFileOperation {
        self.raw.cast()
    }
}

impl Drop for ComPtr {
    fn drop(&mut self) {
        if self.raw.is_null() {
            return;
        }
        // SAFETY: every pointer stored here is an interface pointer returned by a
        // successful COM call; IUnknown is the first three vtable entries of both
        // IShellItem and IFileOperation.
        unsafe {
            let vtable = *(self.raw as *const *const IUnknownVtbl);
            ((*vtable).release)(self.raw);
        }
    }
}

struct ComApartment {
    should_uninitialize: bool,
}

impl ComApartment {
    fn initialize() -> Result<Self, HRESULT> {
        // SAFETY: null reserved pointer and a documented apartment flag.
        let result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
        if result < 0 && result != RPC_E_CHANGED_MODE {
            return Err(result);
        }
        Ok(Self {
            should_uninitialize: result == S_OK || result == S_FALSE,
        })
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.should_uninitialize {
            // SAFETY: this call balances the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
        }
    }
}

enum PreparationFailure {
    Unavailable(io::Error),
    Operation(OperationError),
}

impl PreparationFailure {
    fn may_retry_with_legacy_shell(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

const IID_ISHELL_ITEM: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);
const IID_IFILE_OPERATION: GUID = GUID::from_u128(0x947aab5f_0a5c_4c13_b4d6_4bf7836fc9f8);
const CLSID_FILE_OPERATION: GUID = GUID::from_u128(0x3ad05575_8857_4850_9277_11b85bdb8e09);

pub(super) fn delete_to_recycle_bin(path: &Path) -> Result<(), OperationError> {
    match delete_to_recycle_bin_with_com(path) {
        Ok(()) => Ok(()),
        Err(failure @ PreparationFailure::Unavailable(_)) => {
            debug_assert!(failure.may_retry_with_legacy_shell());
            let PreparationFailure::Unavailable(error) = failure else {
                unreachable!("o padrão garante falha de preparação");
            };
            delete_to_recycle_bin_legacy(path, error)
        }
        Err(PreparationFailure::Operation(error)) => Err(error),
    }
}

fn delete_to_recycle_bin_with_com(path: &Path) -> Result<(), PreparationFailure> {
    let _apartment = ComApartment::initialize()
        .map_err(|result| PreparationFailure::Unavailable(io::Error::from_raw_os_error(result)))?;

    let mut file_operation = std::ptr::null_mut();
    // SAFETY: CLSID/IID are static ABI-compatible GUIDs; the output pointer is
    // valid for the duration of this call and receives an owned COM interface.
    let result = unsafe {
        CoCreateInstance(
            &CLSID_FILE_OPERATION,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IFILE_OPERATION,
            &mut file_operation,
        )
    };
    if result < 0 {
        return Err(PreparationFailure::Unavailable(
            io::Error::from_raw_os_error(result),
        ));
    }
    let file_operation = ComPtr::from_raw(file_operation)
        .map_err(|result| PreparationFailure::Unavailable(io::Error::from_raw_os_error(result)))?;

    let mut shell_item = std::ptr::null_mut();
    let wide_path = wide_path(path);
    // SAFETY: `wide_path` is NUL-terminated and remains alive; the output is an
    // owned IShellItem released by `ComPtr`.
    let result = unsafe {
        SHCreateItemFromParsingName(
            wide_path.as_ptr() as PCWSTR,
            std::ptr::null_mut(),
            &IID_ISHELL_ITEM,
            &mut shell_item,
        )
    };
    if result < 0 {
        return Err(PreparationFailure::Unavailable(
            io::Error::from_raw_os_error(result),
        ));
    }
    let shell_item = ComPtr::from_raw(shell_item)
        .map_err(|result| PreparationFailure::Unavailable(io::Error::from_raw_os_error(result)))?;

    let operation = file_operation.as_file_operation();
    // SAFETY: `operation` is a valid IFileOperation interface and the vtable
    // layout mirrors the Windows SDK declaration through GetAnyOperationsAborted.
    let vtable = unsafe { &*(*operation).vtable };
    let result = unsafe { (vtable.set_operation_flags)(operation.cast(), IFILE_OPERATION_FLAGS) };
    if result < 0 {
        return Err(PreparationFailure::Unavailable(
            io::Error::from_raw_os_error(result),
        ));
    }

    // SAFETY: the item and operation are valid COM interfaces. The item is only
    // declared here; no filesystem mutation occurs until PerformOperations.
    let result =
        unsafe { (vtable.delete_item)(operation.cast(), shell_item.raw, std::ptr::null_mut()) };
    if result < 0 {
        return Err(PreparationFailure::Unavailable(
            io::Error::from_raw_os_error(result),
        ));
    }

    // SAFETY: all queued operation state remains owned by the COM object until
    // PerformOperations returns.
    let result = unsafe { (vtable.perform_operations)(operation.cast()) };
    if result < 0 {
        return Err(PreparationFailure::Operation(from_hresult(
            "mover para a Lixeira",
            path,
            result,
        )));
    }

    let mut aborted = 0;
    // SAFETY: `aborted` is a valid output slot and the COM object remains alive.
    let result = unsafe { (vtable.get_any_operations_aborted)(operation.cast(), &mut aborted) };
    if result < 0 {
        return Err(PreparationFailure::Operation(from_hresult(
            "mover para a Lixeira",
            path,
            result,
        )));
    }
    if aborted != 0 {
        return Err(PreparationFailure::Operation(from_io(
            "mover para a Lixeira",
            path,
            io::Error::new(io::ErrorKind::Interrupted, "operação cancelada pelo Shell"),
        )));
    }
    Ok(())
}

fn delete_to_recycle_bin_legacy(
    path: &Path,
    _preparation_error: io::Error,
) -> Result<(), OperationError> {
    let mut source = wide_path(path);
    source.push(0);
    let mut operation = SHFILEOPSTRUCTW {
        hwnd: std::ptr::null_mut(),
        wFunc: FO_DELETE,
        pFrom: source.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOERRORUI | FOF_NOCONFIRMATION | FOF_NORECURSION | FOF_SILENT)
            as u16,
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

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn from_hresult(operation: &'static str, path: &Path, result: HRESULT) -> OperationError {
    from_io(operation, path, io::Error::from_raw_os_error(result))
}

#[cfg(test)]
mod tests {
    use super::{
        FOFX_RECYCLEONDELETE, IFILE_OPERATION_FLAGS, IID_IFILE_OPERATION, IID_ISHELL_ITEM,
        PreparationFailure, from_hresult, wide_path,
    };
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[test]
    fn path_buffer_for_shell_item_has_one_terminator() {
        let buffer = wide_path(std::path::Path::new("C:\\Rovex\\ação.txt"));
        assert_eq!(buffer.last(), Some(&0));
        assert_eq!(buffer.iter().filter(|value| **value == 0).count(), 1);
    }

    #[test]
    fn legacy_path_buffer_is_double_null_terminated() {
        let mut buffer = OsStr::new("C:\\Rovex\\arquivo.txt")
            .encode_wide()
            .collect::<Vec<_>>();
        buffer.extend([0, 0]);
        assert_eq!(buffer[buffer.len() - 2..], [0, 0]);
        assert!(!buffer[..buffer.len() - 2].contains(&0));
    }

    #[test]
    fn com_guids_and_flags_match_the_windows_contract() {
        let expected_file_operation =
            windows_sys::core::GUID::from_u128(0x947aab5f_0a5c_4c13_b4d6_4bf7836fc9f8);
        assert_eq!(IID_IFILE_OPERATION.data1, expected_file_operation.data1);
        assert_eq!(IID_IFILE_OPERATION.data2, expected_file_operation.data2);
        assert_eq!(IID_IFILE_OPERATION.data3, expected_file_operation.data3);
        assert_eq!(IID_IFILE_OPERATION.data4, expected_file_operation.data4);
        let expected_shell_item =
            windows_sys::core::GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);
        assert_eq!(IID_ISHELL_ITEM.data1, expected_shell_item.data1);
        assert_eq!(IID_ISHELL_ITEM.data2, expected_shell_item.data2);
        assert_eq!(IID_ISHELL_ITEM.data3, expected_shell_item.data3);
        assert_eq!(IID_ISHELL_ITEM.data4, expected_shell_item.data4);
        assert_ne!(IFILE_OPERATION_FLAGS & FOFX_RECYCLEONDELETE, 0);
    }

    #[test]
    fn only_preparation_failures_allow_legacy_retry() {
        let unavailable =
            PreparationFailure::Unavailable(std::io::Error::other("COM indisponível"));
        let operation = PreparationFailure::Operation(crate::operations::OperationError::Cancelled);
        assert!(unavailable.may_retry_with_legacy_shell());
        assert!(!operation.may_retry_with_legacy_shell());
    }

    #[test]
    fn hresult_is_preserved_as_structured_filesystem_error() {
        let error = from_hresult(
            "mover para a Lixeira",
            std::path::Path::new("C:\\Rovex\\arquivo.txt"),
            windows_sys::Win32::Foundation::E_FAIL,
        );
        assert!(matches!(
            error,
            crate::operations::OperationError::FileSystem {
                raw_os_error: Some(windows_sys::Win32::Foundation::E_FAIL),
                ..
            }
        ));
    }
}
