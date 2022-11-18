use crate::{fcntl_update_flags, pty::Pty};
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use nix::{
    errno::Errno,
    fcntl::OFlag,
    sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg},
    unistd::{read, write},
};
use std::{
    io,
    os::{fd::AsRawFd, unix::io::RawFd},
};

pub struct PtyHandler<H: EventHandler> {
    pty: Pty,
    stdin: RawFd,
    stdout: RawFd,
    event_handler: H,
    poll: Poll,
    events: Events,
    buffer: Box<[u8; 1024]>,
}

impl<E: EventHandler> PtyHandler<E> {
    const STDIN_TOKEN: Token = Token(0);
    const PTY_TOKEN: Token = Token(1);

    pub fn new(pty: Pty, stdin: RawFd, stdout: RawFd, event_handler: E) -> Result<Self, io::Error> {
        fcntl_update_flags(stdin, |flags| flags | OFlag::O_NONBLOCK)?;

        let poll = Poll::new()?;
        let registry = poll.registry();

        registry.register(&mut SourceFd(&stdin), Self::STDIN_TOKEN, Interest::READABLE)?;
        registry.register(
            &mut SourceFd(&pty.as_raw_fd()),
            Self::PTY_TOKEN,
            Interest::READABLE,
        )?;

        Ok(Self {
            pty,
            stdin,
            stdout,
            event_handler,
            poll,
            events: Events::with_capacity(1024),
            buffer: Box::from([0; 1024]),
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut termios = tcgetattr(self.stdin)?;
        let old_termios = termios.clone();
        cfmakeraw(&mut termios);
        tcsetattr(self.stdin, SetArg::TCSANOW, &termios)?;
        'outer: loop {
            self.poll.poll(&mut self.events, None)?;
            for event in &self.events {
                match event.token() {
                    Self::STDIN_TOKEN => match read(self.stdin, &mut self.buffer[..]) {
                        Ok(count) => {
                            if let Some(slice) = self
                                .event_handler
                                .handle(Event::Input(&mut self.buffer[..count]))
                            {
                                write_all(self.pty.as_raw_fd(), slice)?;
                            }
                        }

                        Err(Errno::EWOULDBLOCK) => break,
                        Err(Errno::EIO) => break 'outer,
                        Err(e) => Err(e)?,
                    },
                    Self::PTY_TOKEN => match read(self.pty.as_raw_fd(), &mut self.buffer[..]) {
                        Ok(count) => {
                            if let Some(slice) = self
                                .event_handler
                                .handle(Event::Output(&mut self.buffer[..count]))
                            {
                                write_all(self.stdout, slice)?
                            }
                        }
                        Err(Errno::EWOULDBLOCK) => break,
                        Err(Errno::EIO) => break 'outer,
                        Err(e) => Err(e)?,
                    },
                    _ => unreachable!("invalid event token: {event:?}"),
                }
            }
        }
        tcsetattr(self.stdin, SetArg::TCSANOW, &old_termios)?;
        Ok(())
    }
}

pub trait EventHandler {
    fn handle<'a>(&mut self, event: Event<'a>) -> Option<&'a [u8]>;
}

pub enum Event<'a> {
    Input(&'a mut [u8]),
    Output(&'a mut [u8]),
}

fn write_all(fd: RawFd, buf: &[u8]) -> Result<(), Errno> {
    let mut offset = 0;
    while offset < buf.len() {
        offset += write(fd, &buf[offset..])?;
    }
    Ok(())
}
