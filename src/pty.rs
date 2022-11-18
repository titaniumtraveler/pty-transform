use crate::fcntl_update_flags;
use nix::{
    errno::Errno,
    fcntl::OFlag,
    pty::{forkpty, ForkptyResult},
    unistd::{execvp, ForkResult, Pid},
};
use std::{
    ffi::CStr,
    os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd},
};

#[derive(Debug)]
pub struct Pty {
    pty: OwnedFd,
}

impl Pty {
    pub fn new<S: AsRef<CStr>>(command: &CStr, args: &[S]) -> Result<Self, Errno> {
        let (pty, _) = spawn_pty(command, args)?;
        fcntl_update_flags(pty.as_raw_fd(), |flags| flags | OFlag::O_NONBLOCK)?;
        Ok(Self { pty })
    }
}

impl AsRawFd for Pty {
    fn as_raw_fd(&self) -> RawFd {
        self.pty.as_raw_fd()
    }
}

fn spawn_pty<S: AsRef<CStr>>(command: &CStr, args: &[S]) -> Result<(OwnedFd, Pid), Errno> {
    let ForkptyResult {
        master,
        fork_result,
        // Safety: Only execvp is called in `ForkResult::Child`. Therefore it's safe.
    } = unsafe { forkpty(None, None)? };

    match fork_result {
        ForkResult::Child => {
            execvp(command, args)?;
            unreachable!()
        }
        ForkResult::Parent { child } => {
            Ok((
                // SAFETY: forkpty returns an open and owned fd. Therefore it's safe.
                unsafe { OwnedFd::from_raw_fd(master) },
                child,
            ))
        }
    }
}
