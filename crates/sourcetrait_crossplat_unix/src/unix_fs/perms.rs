use crate::*;
use super::prelude::*;

/// [man page](https://man7.org/linux/man-pages/man2/lchown.2.html)
pub fn chown<P>(dst: P, uid: Option<UID>, gid: Option<GID>, opts: &FsOptions) -> cross::BridgeResult<()>
where
    P: AsRef<Path>,
{
    let uid = uid.unwrap_or_default();
    let gid = gid.unwrap_or_default();
    
    if uid == 0 && gid == 0 {
        return Ok(());
    }
    
    let dst = dst.as_ref();    
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|_| cross::BridgeError::String)?;
    
    let code = unsafe {
        match opts.follow_symlinks {
            true => libc::chown(
                dst_cstr.as_ptr(),
                uid,
                gid,
            ),
            false => libc::lchown(
                dst_cstr.as_ptr(),
                uid,
                gid,
            ),
        }
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::chown.bridge_lasterr(&opts),
        _ => Er::chown.bridge_err_unknown(opts),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/chmod.2.html)
pub fn chmod<P>(dst: P, mode: UnixFileMode, opts: &FsOptions) -> cross::BridgeResult<()>
where
    P: AsRef<Path>,
{
    let flags = match opts.follow_symlinks {
        true => 0,
        false => libc::AT_SYMLINK_NOFOLLOW,
    };
    
    let dst = dst.as_ref();    
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|_| cross::BridgeError::String)?;
    
    let code = unsafe {
        libc::fchmodat(
            libc::AT_FDCWD,
            dst_cstr.as_ptr(),
            mode,
            flags,
        )
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::fchmodat.bridge_lasterr_unsupported_ok_if(!opts.follow_symlinks, opts),
        _ => Er::fchmodat.bridge_err_unknown(opts),
    }
}

