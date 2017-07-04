// Taken from https://github.com/hibariya/pty-rs
// TODO: Licensing?

extern crate libc;

#[link(name = "c")]
extern {
    pub fn posix_openpt(flags: libc::c_int) -> libc::c_int;
    pub fn grantpt(fd: libc::c_int) -> libc::c_int;
    pub fn unlockpt(fd: libc::c_int) -> libc::c_int;
    pub fn ptsname(fd: libc::c_int) -> *mut libc::c_schar;
}
