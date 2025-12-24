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

pub unsafe fn cstd_across_getxattr(
    path: *const libc::c_char,
    name: *const libc::c_char,
    value: *mut libc::c_void,
    size: libc::size_t,
) -> libc::ssize_t {
    unsafe { imp::across_getxattr(
        path,
        name,
        value,
        size,
    ) }
}
