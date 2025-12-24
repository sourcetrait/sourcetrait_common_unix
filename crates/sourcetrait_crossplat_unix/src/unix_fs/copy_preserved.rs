use crate::*;
use super::prelude::*;

/// Copies a file while preserving all extended metadata:
/// - Ownership
/// - Permissions
/// - Created and modified timestamps
/// - Extended attributes
/// 
/// Operates directly on symbolic links (does not follow).
/// 
/// Note: `chmod` is *usually* ignored for symbolic links. Outliers:
/// - Various BSD variants, including MacOS
/// - Solaris variants
/// - AIX variants 
pub fn copy_preserved<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, opts: &FsOptions) -> Result<(), CopyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    let metadata = match opts.follow_symlinks {
            false if src.is_symlink() => src.symlink_metadata(),
            _ => src.metadata(),
        }
        .map_err(|e| CopyError::clean(&src, &dst, e))?;
    
    let src_cstr = CString::new(src.as_os_str().as_bytes())
        .map_err(|e| CopyError::clean(&src, &dst, io::Error::new(io::ErrorKind::InvalidInput, e)))?;
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|e| CopyError::clean(&src, &dst, io::Error::new(io::ErrorKind::InvalidInput, e)))?;
    
    match opts.follow_symlinks {
        true => {
            fs::copy(&src, &dst)
                .map_err(|e| recover(&src, &dst, e))?;
        },
        false => {
            std::os::unix::fs::symlink(&src, &dst)
                .map_err(|e| recover(&src, &dst, e))?;
        },
    }
    
    chown(&dst_cstr, &metadata, &opts) // ownership
        .map_err(|e| recover(&src, &dst, e))?;
    
    chmod(&dst_cstr, &metadata, &opts) // permissions
        .map_err(|e| recover(&src, &dst, e))?;
    
    copy_xattrs(&src_cstr, &dst_cstr, &opts) // extended attributes
        .map_err(|e| recover(src, dst, e))?;
    
    set_timestamps(&dst_cstr, &metadata, &opts) // accessed, modified 
        .map_err(|e| recover(&src, &dst, e))?;
    
    Ok(())
}

/// [man page](https://man7.org/linux/man-pages/man2/lchown.2.html)
fn chown(path: &CStr, meta: &fs::Metadata, opts: &FsOptions) -> io::Result<()> {
    let code = unsafe {
        match opts.follow_symlinks {
            true => libc::chown(
                path.as_ptr(),
                meta.uid(),
                meta.gid(),
            ),
            false => libc::lchown(
                path.as_ptr(),
                meta.uid(),
                meta.gid(),
            ),
        }
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::chown.lasterr(&opts),
        _ => Er::chown.err_unknown(opts),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/chmod.2.html)
fn chmod(path: &CStr, meta: &fs::Metadata, opts: &FsOptions) -> io::Result<()> {
    let flags = match opts.follow_symlinks {
        true => 0,
        false => libc::AT_SYMLINK_NOFOLLOW,
    };
    
    let code = unsafe {
        libc::fchmodat(
            libc::AT_FDCWD,
            path.as_ptr(),
            meta.mode() as libc::mode_t,
            flags,
        )
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::fchmodat.lasterr_unsupported_ok_if(!opts.follow_symlinks, opts),
        _ => Er::fchmodat.err_unknown(opts),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/utimensat.2.html)
fn set_timestamps(dst: &CStr, meta: &fs::Metadata, opts: &FsOptions) -> io::Result<()> {
    let times = [
        libc::timespec {
            tv_sec: meta.atime(),
            tv_nsec: meta.atime_nsec(),
        },
        libc::timespec {
            tv_sec: meta.mtime(),
            tv_nsec: meta.mtime_nsec(),
        },
    ];
    
    let flags = match opts.follow_symlinks {
        true => 0,
        false => libc::AT_SYMLINK_NOFOLLOW,
    };
    
    let code = unsafe {
        libc::utimensat(
            libc::AT_FDCWD,
            dst.as_ptr(),
            times.as_ptr(),
            flags,
        )
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::utimensat.lasterr(opts),
        _ => Er::utimensat.err_unknown(opts),
    }
}

/// - [man page: listxattr](https://man7.org/linux/man-pages/man2/listxattr.2.html)  
/// - [man page: getxattr](https://man7.org/linux/man-pages/man2/getxattr.2.html)
/// - [man page: setxattr](https://man7.org/linux/man-pages/man2/setxattr.2.html)
fn copy_xattrs(src: &CStr, dst: &CStr, opts: &FsOptions) -> io::Result<()> {
    let code = unsafe {
        match opts.follow_symlinks {
            true => cstd_across_listxattr(
                src.as_ptr(),
                ptr::null_mut(),
                0,
            ),
            false => cstd_across_llistxattr(
                src.as_ptr(),
                ptr::null_mut(),
                0,
            ),
        }
    };
    
    let size = match code {
        n if n > 0 => n,
        0 => return Ok(()),
        -1 => return Er::listxattr.lasterr_unsupported_ok(opts),
        _ => return Er::listxattr.err_unknown(&opts),
    };
        
    let mut list = vec![0u8; size as libc::size_t];
    let code = unsafe {
        match opts.follow_symlinks {
            true => cstd_across_listxattr(
                src.as_ptr(),
                list.as_mut_ptr() as *mut libc::c_char,
                size as libc::size_t,
            ),
            false => cstd_across_llistxattr(
                src.as_ptr(),
                list.as_mut_ptr() as *mut libc::c_char,
                size as libc::size_t,
            ),
        }
    };

    match code {
        0 => return Ok(()),
        -1 => return Er::listxattr.lasterr(opts),
        n if n == size => (), // double-check
        _ => return Er::listxattr.err_unknown(&opts),
    };
        
    // each name is null-terminated
    let mut pos = 0;
    let list_len = list.len();
    while pos < list_len {
        let name_start = pos;
        while pos < list_len && list[pos] != 0 {
            pos += 1;
        }
        
        if pos <= name_start {
            break;
        }
        
        let name = CStr::from_bytes_with_nul(&list[name_start..=pos])
            .map_err(|_| Er::listxattr.data(opts))?;
        
        let code = unsafe {
            match opts.follow_symlinks {
                true => cstd_across_getxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    ptr::null_mut(),
                    0,
                ),
                false => cstd_across_lgetxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    ptr::null_mut(),
                    0,
                ),
            }
        };
        
        let value_size = match code {
            n if n > 0 => n,
            -1 => return Er::getxattr.lasterr(opts),
            _ => return Er::getxattr.err_unknown(&opts),
        };
        
        let mut value = vec![0u8; value_size as libc::size_t];
        let code = unsafe {
            match opts.follow_symlinks {
                true => cstd_across_getxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value_size as libc::size_t,
                ),
                false => cstd_across_lgetxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value_size as libc::size_t,
                ),
            }
        };
        
        match code {
            n if n == value_size => (), // double-check
            -1 => return Er::getxattr.lasterr(opts),
            _ => return Er::getxattr.err_unknown(&opts),
        };
        
        let code = unsafe {
            match opts.follow_symlinks {
                true => cstd_across_setxattr(
                    dst.as_ptr(),
                    name.as_ptr(),
                    value.as_ptr() as *const libc::c_void,
                    value.len(),
                    0,
                ),
                false => cstd_across_lsetxattr(
                    dst.as_ptr(),
                    name.as_ptr(),
                    value.as_ptr() as *const libc::c_void,
                    value.len(),
                    0,
                ),
            }
        };
        
        match code {
            0 => (),
            -1 => Er::setxattr.lasterr_unsupported_ok_if(opts.lossy_extended_attributes, opts)?,
            _ => return Er::setxattr.err_unknown(&opts),
        };
        
        pos += 1;
    }
    
    Ok(())
}

fn recover<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, source: io::Error) -> CopyError {
    let src = src.as_ref().to_path_buf();
    let dst = dst.as_ref().to_path_buf();
    
    if fs::exists(&dst).is_ok_and(|exists| !exists) {
        return CopyError::Clean { src, dst, source };
    }
    
    match fs::remove_file(&dst) {
        Ok(_) => CopyError::Clean { src, dst, source },
        Err(recover_source) => CopyError::Dirty { src, dst, source, recover_source }
    }
}
