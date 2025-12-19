use crate::*;
use er::Er;

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
    
    set_timestamps(&dst_cstr, &metadata, &opts) // created, modified 
        .map_err(|e| recover(&src, &dst, e))?;
    
    copy_xattrs(&src_cstr, &dst_cstr, &opts) // extended attributes
        .map_err(|e| recover(src, dst, e))?;
    
    Ok(())
}

/// [man page](https://man7.org/linux/man-pages/man2/lchown.2.html)
fn chown(path: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
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
fn chmod(path: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
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
        -1 => Er::fchmodat.lasterr_unsupported_ok_if(!opts.follow_symlinks, opts),
        _ => Er::fchmodat.err_unknown(opts),
    }
}

/// [man page](https://man7.org/linux/man-pages/man2/utimensat.2.html)
fn set_timestamps(dst: &CStr, meta: &fs::Metadata, opts: &CopyOptions) -> io::Result<()> {
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
fn copy_xattrs(src: &CStr, dst: &CStr, opts: &CopyOptions) -> io::Result<()> {
    let code = unsafe {
        match opts.follow_symlinks {
            true => libc::listxattr(
                src.as_ptr(),
                ptr::null_mut(),
                0,
            ),
            false => libc::llistxattr(
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
        
    let mut list = vec![0u8; size as usize];
    let code = unsafe {
        match opts.follow_symlinks {
            true => libc::listxattr(
                src.as_ptr(),
                list.as_mut_ptr() as *mut i8,
                size as usize,
            ),
            false => libc::llistxattr(
                src.as_ptr(),
                list.as_mut_ptr() as *mut i8,
                size as usize,
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
                true => libc::getxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    ptr::null_mut(),
                    0,
                ),
                false => libc::lgetxattr(
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
        
        let mut value = vec![0u8; value_size as usize];
        let code = unsafe {
            match opts.follow_symlinks {
                true => libc::getxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value_size as usize,
                ),
                false => libc::lgetxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value_size as usize,
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
                true => libc::setxattr(
                    dst.as_ptr(),
                    name.as_ptr(),
                    value.as_ptr() as *const libc::c_void,
                    value.len(),
                    0,
                ),
                false => libc::lsetxattr(
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

pub(super) mod er {
    use crate::*;
    const CHOWN: &'static str = "libc::chown";
    const LCHOWN: &'static str = "libc::lchown";
    const FCHMODAT: &'static str = "libc::fchmodat";
    const GETXATTR: &'static str = "libc::getxattr";
    const LGETXATTR: &'static str = "libc::lgetxattr";
    const LLISTXATTR: &'static str = "libc::llistxattr";
    const LISTXATTR: &'static str = "libc::listxattr";
    const LSETXATTR: &'static str = "libc::lsetxattr";
    const SETXATTR: &'static str = "libc::setxattr";
    const UTIMENSAT: &'static str = "libc::utimensat";
    
    const E_UNKNOWN: &'static str = "Unknown error from ";
    const UNKNOWN_CHOWN: &'static str = concatcp!(E_UNKNOWN, CHOWN); 
    const UNKNOWN_LCHOWN: &'static str = concatcp!(E_UNKNOWN, LCHOWN); 
    const UNKNOWN_FCHMODAT: &'static str = concatcp!(E_UNKNOWN, FCHMODAT); 
    const UNKNOWN_GETXATTR: &'static str = concatcp!(E_UNKNOWN, GETXATTR); 
    const UNKNOWN_LGETXATTR: &'static str = concatcp!(E_UNKNOWN, LGETXATTR); 
    const UNKNOWN_LISTXATTR: &'static str = concatcp!(E_UNKNOWN, LISTXATTR); 
    const UNKNOWN_LLISTXATTR: &'static str = concatcp!(E_UNKNOWN, LLISTXATTR); 
    const UNKNOWN_SETXATTR: &'static str = concatcp!(E_UNKNOWN, SETXATTR); 
    const UNKNOWN_LSETXATTR: &'static str = concatcp!(E_UNKNOWN, LSETXATTR); 
    const UNKNOWN_UTIMENSAT: &'static str = concatcp!(E_UNKNOWN, UTIMENSAT); 
    
    #[allow(non_camel_case_types)]
    pub(super) enum Er {
        chown,
        fchmodat,
        listxattr,
        getxattr,
        setxattr,
        utimensat,
    }
    
    impl Er {
        #[inline]
        const fn e_unknown(self, opts: &super::CopyOptions) -> &'static str {
            match self {
                Self::chown => match opts.follow_symlinks {
                    true => UNKNOWN_CHOWN,
                    false => UNKNOWN_LCHOWN,
                },
                Self::fchmodat => UNKNOWN_FCHMODAT,
                Self::listxattr => match opts.follow_symlinks {
                    true => UNKNOWN_LISTXATTR,
                    false => UNKNOWN_LLISTXATTR,
                },
                Self::getxattr => match opts.follow_symlinks {
                    true => UNKNOWN_GETXATTR,
                    false => UNKNOWN_LGETXATTR,
                },
                Self::setxattr => match opts.follow_symlinks {
                    true => UNKNOWN_SETXATTR,
                    false => UNKNOWN_LSETXATTR,
                },
                Self::utimensat => UNKNOWN_UTIMENSAT,
            }
        }
        
        #[inline]
        pub(super) fn err_unknown<T>(self, opts: &super::CopyOptions) -> io::Result<T> {
            io::Result::Err(io::Error::new(io::ErrorKind::Other, self.e_unknown(opts)))
        }
        
        #[inline]
        pub(super) fn lasterr<T>(self, opts: &super::CopyOptions) -> io::Result<T> {
            self._err(opts)
        }
        
        pub(super) fn lasterr_unsupported_ok(self, opts: &super::CopyOptions) -> io::Result<()> {
            self.lasterr_unsupported_ok_if(false, opts)
        }
        
        pub(super) fn lasterr_unsupported_ok_if(self, unsupported_ok: bool, opts: &super::CopyOptions) -> io::Result<()> {
            let err = io::Error::last_os_error();
            return match err.raw_os_error() {
                Some(libc::ENOTSUP) if unsupported_ok => Ok(()),
                Some(_) => Err(err),
                None => self.err_unknown(opts),
            }
        }

        fn _err<T>(self, opts: &super::CopyOptions) -> io::Result<T> {
            let err = io::Error::last_os_error();
            return match err.raw_os_error() {
                Some(_) => Err(err),
                None => self.err_unknown(opts),
            }
        }
        
        pub(super) fn data(self, opts: &super::CopyOptions) -> io::Error {
            io::Error::new(io::ErrorKind::InvalidData, self.e_unknown(opts))
        }
    }
}
