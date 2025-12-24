//use crate::*;

pub(in super) unsafe fn across_lsetxattr(
    path: *const libc::c_char,
    name: *const libc::c_char,
    value: *const libc::c_void,
    size: libc::size_t,
    flags: libc::c_int,
) -> libc::c_int {
    unsafe { libc::setxattr(
        path,
        name,
        value,
        size,
        0, // position
        flags | libc::XATTR_NOFOLLOW,
    ) }
}



