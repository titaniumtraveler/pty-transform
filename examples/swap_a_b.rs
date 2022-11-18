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
    let mut pty_handler = PtyHandler::new(pty, stdin().as_raw_fd(), stdout().as_raw_fd(), SwapAB)?;
    pty_handler.run()?;

    Ok(())
}

struct SwapAB;

impl EventHandler for SwapAB {
    fn handle<'a>(&mut self, event: Event<'a>) -> Option<&'a [u8]> {
        match event {
            Event::Input(slice) => {
                for byte in &mut *slice {
                    match byte {
                        b'a' => {
                            *byte = b'b';
                        }
                        b'b' => {
                            *byte = b'a';
                        }
                        _ => (),
                    }
                }
                Some(slice)
            }
            Event::Output(slice) => Some(slice),
        }
    }
}
