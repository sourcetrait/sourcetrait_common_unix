//! Attempts to provide standardized libc calls using Linux's implementation
//! as the common denominator.
//use crate::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use self::linux as imp;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use self::macos as imp;

pub unsafe fn cstd_across_lsetxattr(
    path: *const libc::c_char,
    name: *const libc::c_char,
    value: *const libc::c_void,
    size: libc::size_t,
    flags: libc::c_int,
) -> libc::c_int {
    unsafe { imp::across_lsetxattr(
        path,
        name,
        value,
        size,
        flags
    ) }
}
