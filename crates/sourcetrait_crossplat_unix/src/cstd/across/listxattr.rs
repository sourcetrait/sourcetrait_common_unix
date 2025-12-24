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

pub unsafe fn cstd_across_listxattr(
    path: *const libc::c_char,
    list: *mut libc::c_char,
    size: libc::size_t
) -> libc::ssize_t {
    unsafe { imp::across_listxattr(
        path,
        list,
        size,
    ) }
}
