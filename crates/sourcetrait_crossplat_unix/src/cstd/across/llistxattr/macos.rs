//use crate::*;

pub(in super) unsafe fn across_llistxattr(
    path: *const libc::c_char,
    list: *mut libc::c_char,
    size: libc::size_t
) -> libc::ssize_t {
    unsafe { libc::listxattr(
        path,
        list,
        size,
        libc::XATTR_NOFOLLOW, // options
    ) }
}
