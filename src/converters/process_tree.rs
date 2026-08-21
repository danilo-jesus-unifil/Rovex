use std::io;
use std::process::{Child, Command};

#[cfg(unix)]
pub(crate) fn configure_command(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
pub(crate) fn configure_command(_command: &mut Command) {}

#[cfg(unix)]
pub(crate) struct ProcessTree {
    process_group: libc::pid_t,
}

#[cfg(unix)]
impl ProcessTree {
    pub(crate) fn attach(child: &Child) -> io::Result<Self> {
        let process_group = libc::pid_t::try_from(child.id()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "PID do processo fora do limite",
            )
        })?;
        if process_group <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "PID do processo inválido",
            ));
        }
        Ok(Self { process_group })
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        // `process_group(0)` torna o PID do filho o ID do grupo, portanto o
        // sinal não alcança o processo Rovex nem grupos não relacionados.
        let result = unsafe { libc::killpg(self.process_group, libc::SIGKILL) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(windows)]
pub(crate) struct ProcessTree {
    job: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl ProcessTree {
    pub(crate) fn attach(child: &Child) -> io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };

        // SAFETY: null security attributes/name request an unnamed private job.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }

        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `limits` is the documented structure for this information class.
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            // SAFETY: job was created by this function and is closed exactly once here.
            unsafe { CloseHandle(job) };
            return Err(io::Error::last_os_error());
        }

        // SAFETY: the child handle is valid while `child` is alive; assigning it
        // does not transfer ownership of the process handle to the job.
        let assigned = unsafe { AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) };
        if assigned == 0 {
            // SAFETY: job was created by this function and is closed exactly once here.
            unsafe { CloseHandle(job) };
            return Err(io::Error::last_os_error());
        }
        Ok(Self { job })
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;
        // SAFETY: `job` is owned by this instance and remains valid until Drop.
        let result = unsafe { TerminateJobObject(self.job, 1) };
        if result != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        // SAFETY: `job` is owned by this instance and closed exactly once.
        unsafe { CloseHandle(self.job) };
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) struct ProcessTree;

#[cfg(not(any(unix, windows)))]
impl ProcessTree {
    pub(crate) fn attach(_child: &Child) -> io::Result<Self> {
        Ok(Self)
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        Ok(())
    }
}
