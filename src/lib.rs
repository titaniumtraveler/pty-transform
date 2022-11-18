#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::missing_safety_doc)]

pub use crate::{
    event_handler::{Event, EventHandler, PtyHandler},
    pty::Pty,
};

use nix::{
    errno::Errno,
    fcntl::{fcntl, FcntlArg, OFlag},
};
use std::os::unix::prelude::RawFd;

mod event_handler;
mod pty;

fn fcntl_update_flags<F>(fd: RawFd, update_flags: F) -> Result<(), Errno>
where
    F: FnOnce(OFlag) -> OFlag,
{
    let flags = fcntl(fd, FcntlArg::F_GETFL)?;
    // SAFETY: The flags returned from `fcntl(fd, FcntlArg::F_GETFL)` are valid flags.
    let flags = unsafe { OFlag::from_bits_unchecked(flags) };
    fcntl(fd, FcntlArg::F_SETFL(update_flags(flags)))?;
    Ok(())
}
