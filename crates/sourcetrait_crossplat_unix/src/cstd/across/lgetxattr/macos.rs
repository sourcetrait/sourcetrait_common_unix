//use crate::*;

pub(in super) unsafe fn across_lgetxattr(
    path: *const libc::c_char,
    name: *const libc::c_char,
    value: *mut libc::c_void,
    size: libc::size_t,
) -> libc::ssize_t {
    unsafe { libc::getxattr(
        path,
        name,
        value,
        size,
        0, // position
        libc::XATTR_NOFOLLOW, // flags
    ) }
}
