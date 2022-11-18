use pty_transform::{Event, EventHandler, Pty, PtyHandler};
use std::{
    error::Error,
    ffi::{CStr, CString},
    io::{stdin, stdout},
    os::fd::AsRawFd,
};

fn main() -> Result<(), Box<dyn Error>> {
    let command = CString::new(String::from("bash")).unwrap();
    let pty = Pty::new::<&CStr>(&command, &[])?;
    let mut pty_handler = PtyHandler::new(pty, stdin().as_raw_fd(), stdout().as_raw_fd(), NoOp)?;
    pty_handler.run()?;

    Ok(())
}

struct NoOp;

impl EventHandler for NoOp {
    fn handle<'a>(&mut self, event: Event<'a>) -> Option<&'a [u8]> {
        match event {
            Event::Input(slice) | Event::Output(slice) => Some(slice),
        }
    }
}
