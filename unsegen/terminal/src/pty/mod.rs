// Adapted and modified from https://github.com/hibariya/pty-rs
// TODO: Licensing?

#![deny(unstable_features,
        unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

use nix::errno;
use std::fmt;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::result;
use libc;
use std::ffi::OsStr;

mod ffi;

macro_rules! unsafe_try {
    ( $x:expr ) => {{
        let ret = unsafe { $x };

        if ret < 0 {
            return Err(last_error());
        } else {
            ret
        }
    }};
}

#[derive(Debug)]
pub enum Error {
    Sys(i32),
}

pub type Result<T> = result::Result<T, Error>;

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Sys(n) => errno::from_i32(n).desc(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl From<i32> for Error {
    fn from(n: i32) -> Error {
        Error::Sys(n)
    }
}

impl From<::nix::Error> for Error {
    fn from(e: ::nix::Error) -> Error {
        Error::Sys(e.errno() as i32)
    }
}

fn last_error() -> Error {
    Error::from(errno::errno())
}

/// A type representing a pty.
pub struct PTY {
    fd: libc::c_int,
}

use std::sync::{Arc, Mutex};
pub struct PTYInput {
    pty: Arc<Mutex<PTY>>,
}

pub struct PTYOutput {
    pty: Arc<Mutex<PTY>>,
}


impl PTY {
    pub fn open() -> Result<PTY> {
        open_ptm().map(|fd| {
            PTY {
                fd: fd,
            }
        })
    }

    pub fn name(&self) -> &OsStr {
        // man ptsname:
        // "On success, ptsname() returns a pointer to a string in _static_ storage which
        // will be overwritten by subsequent calls. This pointer must not be freed."
        let pts_name = unsafe { ffi::ptsname(self.fd) };

        // This should not happen, as fd is always valid from open to drop.
        assert!((pts_name as *const i32) != ::std::ptr::null(),
                format!("ptsname failed. ({})", last_error()));

        let pts_name_cstr = unsafe { ::std::ffi::CStr::from_ptr(pts_name) };
        let pts_name_slice = pts_name_cstr.to_bytes();

        use ::std::os::unix::ffi::OsStrExt;
        OsStr::from_bytes(pts_name_slice)
    }

    pub fn split_io(self) -> (PTYInput, PTYOutput) {
        let read = Arc::new(Mutex::new(self));
        let write = read.clone();
        ( PTYInput { pty: read }, PTYOutput {pty: write} )
    }
}

impl Drop for PTY {
    fn drop(&mut self) {
        assert!( unsafe { libc::close(self.as_raw_fd()) } == 0,
                 format!("Closing PTY failed ({}).", last_error()));
    }
}

impl AsRawFd for PTY {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Read for PTY {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd, buf)
    }
}

impl Write for PTY {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.fd, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for PTYOutput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        //Panics while reading/writing should not happen
        let fd = self.pty.lock().expect("lock pty for read").fd;
        read(fd, buf)
    }
}

impl Write for PTYInput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        //Panics while reading/writing should not happen
        let fd = self.pty.lock().expect("lock pty for write").fd;
        write(fd, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn open_ptm() -> Result<libc::c_int> {
    let pty_master = unsafe_try!(ffi::posix_openpt(libc::O_RDWR));

    unsafe_try!(ffi::grantpt(pty_master));
    unsafe_try!(ffi::unlockpt(pty_master));

    Ok(pty_master)
}

fn read(fd: libc::c_int, buf: &mut [u8]) -> io::Result<usize> {
    let nread = unsafe {
        libc::read(fd,
                   buf.as_mut_ptr() as *mut libc::c_void,
                   buf.len() as usize)
    };

    if nread < 0 {
        //Ok(0)
        //panic!("read: {:?}", io::Error::last_os_error());
        Err(io::Error::last_os_error())
    } else {
        Ok(nread as usize)
    }
}

fn write(fd: libc::c_int, buf: &[u8]) -> io::Result<usize> {
    let ret = unsafe {
        libc::write(fd,
                    buf.as_ptr() as *const libc::c_void,
                    buf.len() as usize)
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}
