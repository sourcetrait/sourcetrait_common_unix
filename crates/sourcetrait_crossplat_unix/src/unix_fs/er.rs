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
pub(crate) enum Er {
    chown,
    fchmodat,
    listxattr,
    getxattr,
    setxattr,
    utimensat,
}

impl Er {
    #[inline]
    const fn e_unknown(self, opts: &FsOptions) -> &'static str {
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
    
    const fn bridge_err(&self) -> cross::BridgeErr {
        match self {
            Self::chown => cross::BridgeErr::File,
            Self::fchmodat => cross::BridgeErr::File,
            Self::listxattr => cross::BridgeErr::File,
            Self::getxattr => cross::BridgeErr::File,
            Self::setxattr => cross::BridgeErr::File,
            Self::utimensat => cross::BridgeErr::File,
        }
    }
    
    #[inline]
    pub(crate) fn err_unknown<T>(self, opts: &FsOptions) -> io::Result<T> {
        io::Result::Err(io::Error::new(io::ErrorKind::Other, self.e_unknown(opts)))
    }
    
    #[inline]
    pub(crate) fn bridge_err_unknown<T>(self, opts: &FsOptions) -> cross::BridgeResult<T> {
        let noun = self.bridge_err();
        self.err_unknown(opts)
            .map_err(|e| cross::BridgeError::sys_call(noun, e))
    }

    #[inline]
    pub(crate) fn lasterr<T>(self, opts: &FsOptions) -> io::Result<T> {
        self._err(opts)
    }
    
    pub(crate) fn bridge_lasterr<T>(self, opts: &FsOptions) -> cross::BridgeResult<T> {
        let noun = self.bridge_err();
        self._err(opts)
            .map_err(|e| cross::BridgeError::sys_call(noun, e))
    }
    
    pub(crate) fn lasterr_unsupported_ok(self, opts: &FsOptions) -> io::Result<()> {
        self.lasterr_unsupported_ok_if(true, opts)
    }
    
    pub(crate) fn bridge_lasterr_unsupported_ok_if(self, unsupported_ok: bool, opts: &FsOptions) -> cross::BridgeResult<()> {
        let noun = self.bridge_err();
        self.lasterr_unsupported_ok_if(unsupported_ok, opts)
            .map_err(|e| cross::BridgeError::sys_call(noun, e))
    }
        
    pub(crate) fn lasterr_unsupported_ok_if(self, unsupported_ok: bool, opts: &FsOptions) -> io::Result<()> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOTSUP) if unsupported_ok => Ok(()),
            Some(_) => Err(err),
            None => self.err_unknown(opts),
        }
    }
    
    pub(crate) fn lasterr_nodata_ok<T>(self, opts: &FsOptions) -> io::Result<Option<T>> {
        self.lasterr_nodata_ok_if(true, opts)
    }
    
    pub(crate) fn lasterr_nodata_ok_if<T>(self, nodata_ok: bool, opts: &FsOptions) -> io::Result<Option<T>> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENODATA) if nodata_ok => Ok(None),
            Some(_) => Err(err),
            None => self.err_unknown(opts),
        }
    }

    fn _err<T>(self, opts: &FsOptions) -> io::Result<T> {
        let err = io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(_) => Err(err),
            None => self.err_unknown(opts),
        }
    }
    
    pub(crate) fn data(self, opts: &FsOptions) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self.e_unknown(opts))
    }
}
