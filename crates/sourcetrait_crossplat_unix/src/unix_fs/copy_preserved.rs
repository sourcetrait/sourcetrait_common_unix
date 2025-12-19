use crate::*;

/// Options used with [copy_preserved].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CopyOptions {
    /// Dereference symbolic links.
    /// - FALSE: Operates on the symbolic link itself
    /// - TRUE (default): Operates on the actual file linked to
    pub follow_symlinks: bool,
    
    /// Ignore when extended attributes exist for the source path,
    /// but are not supported by destination filesystem.
    /// - FALSE (default): Operation fails and throws an error  
    /// - TRUE: Operation proceeds
    pub lossy_extended_attributes: bool,
}

impl CopyOptions {
    pub const DEFAULT: Self = Self {
        follow_symlinks: true,
        lossy_extended_attributes: false,
    };
    
    pub const FOLLOW_SYMLINKS: bool = true;
    pub const LOSSY_EXT_ATTRIBUTES: bool = false;
}

impl Default for CopyOptions {
    #[inline]
    fn default() -> Self { Self::DEFAULT }
}

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
pub fn copy_preserved<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, opts: &CopyOptions) -> Result<(), CopyError> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    let metadata = src.metadata()
        .map_err(|e| CopyError::clean(&src, &dst, e))?;
    let src_cstr = CString::new(src.as_os_str().as_bytes())
        .map_err(|e| CopyError::clean(&src, &dst, io::Error::new(io::ErrorKind::InvalidInput, e)))?;
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|e| CopyError::clean(&src, &dst, io::Error::new(io::ErrorKind::InvalidInput, e)))?;
    
    fs::copy(&src, &dst)
        .map_err(|e| recover(&src, &dst, e))?;
    
    chown(&dst_cstr, &metadata, &opts) // ownership
        .map_err(|e| recover(&src, &dst, e))?;
    
    chmod(&dst_cstr, &metadata, &opts) // permissions
        .map_err(|e| recover(&src, &dst, e))?;
    
    set_timestamps(&dst_cstr, &metadata) // created, modified 
        .map_err(|e| recover(&src, &dst, e))?;
    
    copy_xattrs(&src_cstr, &dst_cstr, &opts) // extended attributes
        .map_err(|e| recover(src, dst, e))?;
    
    Ok(())
}

/// Describes an IO error that was either cleanly recovered from or not.
/// A clean recovery removed any file that was created.
/// A dirty recovery failed to remove any file that was created. 
#[derive(Debug, snafu::Snafu)]
pub enum CopyError {
    Clean {
        src: PathBuf,
        dst: PathBuf,
        source: io::Error
    },
    Dirty {
        src: PathBuf,
        dst: PathBuf,
        source: io::Error,
        recover_source: io::Error,
    }
}

impl CopyError {
    fn clean<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, source: io::Error) -> Self {
        Self::Clean { src: src.as_ref().into(), dst: dst.as_ref().into(), source }
    }
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

#[inline]
fn err_io_other<T>(msg: &'static str) -> io::Result<T> {
    io::Result::Err(io::Error::new(io::ErrorKind::Other, msg))
}

#[inline]
fn io_data_error(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

/// [man page](https://man7.org/linux/man-pages/man2/lchown.2.html)
fn chown(path: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
    #[inline]
    const fn e_chown_unknown(opts: &CopyOptions) -> &'static str {
        const E_CHOWN_UNKNOWN: &'static str = "Unknown libc::chown error";
        const E_LCHOWN_UNKNOWN: &'static str = "Unknown libc::lchown error";
        
        match opts.follow_symlinks {
            true => E_CHOWN_UNKNOWN,
            false => E_LCHOWN_UNKNOWN,
        }
    }
    
    fn err_chown(opts: &CopyOptions) -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(_) => Err(err),
            None => err_io_other(e_chown_unknown(&opts)),
        }
    }
    
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
        -1 => err_chown(&opts),
        _ => err_io_other(e_chown_unknown(&opts)),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/chmod.2.html)
fn chmod(path: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
    const E_FCHMODAT_UNKNOWN: &'static str = "Unknown libc::fchmodat error";
    
    fn check_fchmodat() -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOTSUP) => Ok(()),
            Some(_) => Err(err),
            None => err_io_other(E_FCHMODAT_UNKNOWN),
        }
    }
    
    let flags = match opts.follow_symlinks {
        true => 0,
        false => libc::AT_SYMLINK_NOFOLLOW,
    };
    
    let code = unsafe {
        libc::fchmodat(
            libc::AT_FDCWD,
            path.as_ptr(),
            meta.mode(),
            flags,
        )
    };
    
    match code {
        0 => Ok(()),
        -1 => check_fchmodat(),
        _ => err_io_other(E_FCHMODAT_UNKNOWN),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/utimensat.2.html)
fn set_timestamps(dst: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
    const E_UTIMENSAT_UNKNOWN: &'static str = "Unknown libc::utimensat error";
    
    fn err_utimensat() -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(_) => Err(err),
            None => err_io_other(E_UTIMENSAT_UNKNOWN),
        }
    }
    
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
        -1 => err_utimensat(),
        _ => err_io_other(E_UTIMENSAT_UNKNOWN),
    }
}

fn copy_xattrs(src: &CStr, dst: &CStr, options: &CopyOptions) -> io::Result<()> {
    const UNSUPPORTED_OK: bool = true;
    const E_LLISTXATTR_UNKNOWN: &'static str = "Unknown libc::llistxattr error";
    const E_LISTXATTR_UNKNOWN: &'static str = "Unknown libc::listxattr error";
    const E_LGETXATTR_UNKNOWN: &'static str = "Unknown libc::lgetxattr error";
    const E_GETXATTR_UNKNOWN: &'static str = "Unknown libc::getxattr error";
    const E_LSETXATTR_UNKNOWN: &'static str = "Unknown libc::lsetxattr error";
    const E_SETXATTR_UNKNOWN: &'static str = "Unknown libc::setxattr error";
    const E_LLISTXATTR_NAME: &'static str = "Invalid name from libc::llistxattr";
    const E_LISTXATTR_NAME: &'static str = "Invalid name from libc::listxattr";
    
    fn err_llistxattr(unsupported_ok: bool) -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOTSUP) if unsupported_ok => Ok(()),
            Some(_) => Err(err),
            None => err_io_other(E_LLISTXATTR_UNKNOWN),
        }
    }
    
    fn err_lgetxattr() -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(_) => Err(err),
            None => err_io_other(E_LGETXATTR_UNKNOWN),
        }
    }
    
    fn check_lsetxattr(unsupported_ok: bool) -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOTSUP) if unsupported_ok => Ok(()),
            Some(_) => Err(err),
            None => err_io_other(E_LSETXATTR_UNKNOWN),
        }
    }

    let code = unsafe {
        libc::llistxattr(
            src.as_ptr(),
            ptr::null_mut(),
            0,
        )
    };
        
    let size = match code {
        -1 => return err_llistxattr(UNSUPPORTED_OK),
        0 => return Ok(()),
        n if n > 0 => n,
        _ => return err_io_other(E_LLISTXATTR_UNKNOWN),
    };
        
    let mut list = vec![0u8; size as usize];
    let code = unsafe {
        libc::llistxattr(
            src.as_ptr(),
            list.as_mut_ptr() as *mut i8,
            size as usize,
        )
    };
        
    match code {
        -1 => return err_llistxattr(!UNSUPPORTED_OK),
        0 => return Ok(()),
        n if n == size => (), // double-check
        _ => return err_io_other(E_LLISTXATTR_UNKNOWN),
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
            .map_err(|_| io_data_error(E_LLISTXATTR_NAME))?;
        
        let code = unsafe {
            libc::lgetxattr(
                src.as_ptr(),
                name.as_ptr(),
                ptr::null_mut(),
                0,
            )
        };
        
        let value_size = match code {
            -1 => return err_lgetxattr(),
            n if n > 0 => n,
            _ => return err_io_other(E_LGETXATTR_UNKNOWN),
        };
        
        let mut value = vec![0u8; value_size as usize];
        let code = unsafe {
            libc::lgetxattr(
                src.as_ptr(),
                name.as_ptr(),
                value.as_mut_ptr() as *mut libc::c_void,
                value_size as usize,
            )
        };
        
        match code {
            -1 => return err_lgetxattr(),
            n if n == value_size => (), // double-check
            _ => return err_io_other(E_LGETXATTR_UNKNOWN),
        };
        
        let code = unsafe {
            libc::lsetxattr(
                dst.as_ptr(),
                name.as_ptr(),
                value.as_ptr() as *const libc::c_void,
                value.len(),
                0,
            )
        };
        
        match code {
            0 => (),
            -1 => check_lsetxattr(options.lossy_extended_attributes)?,
            _ => return err_io_other(E_LSETXATTR_UNKNOWN),
        };
        
        pos += 1;
    }
    
    Ok(())
}
