use crate::*;
use super::prelude::*;

pub fn get_extended_attribute<P: AsRef<Path>, S: AsRef<str>>(src: P, name: S, opts: &FsOptions) -> Result<Option<String>, io::Error> {
    let src = src.as_ref();    
    let src_cstr = CString::new(src.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let name_cstr = CString::new(name.as_ref().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    
    let code = unsafe {
        match opts.follow_symlinks {
            true => cstd_across_getxattr(
                src_cstr.as_ptr(),
                name_cstr.as_ptr(),
                ptr::null_mut(),
                0,
            ),
            false => cstd_across_lgetxattr(
                src_cstr.as_ptr(),
                name_cstr.as_ptr(),
                ptr::null_mut(),
                0,
            ),
        }
    };
    
    let value_size = match code {
        n if n > 0 => n,
        0 => return Ok(Some(String::new())),
        -1 => return Er::getxattr.lasterr_nodata_ok(opts),
        _ => return Er::getxattr.err_unknown(&opts),
    };
    
    let mut value = vec![0u8; value_size as libc::size_t + 1];
    let code = unsafe {
        match opts.follow_symlinks {
            true => cstd_across_getxattr(
                src_cstr.as_ptr(),
                name_cstr.as_ptr(),
                value.as_mut_ptr() as *mut libc::c_void,
                value_size as libc::size_t,
            ),
            false => cstd_across_lgetxattr(
                src_cstr.as_ptr(),
                name_cstr.as_ptr(),
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
    
    let value_cstr = CStr::from_bytes_with_nul(&value[..=value_size as usize])
        .map_err(|_| Er::getxattr.data(opts))?;
    let value = value_cstr.to_str()
        .map_err(|_| Er::getxattr.data(opts))?
        .to_string();
    
    Ok(Some(value))
}

pub fn set_extended_attribute<P: AsRef<Path>, S1: AsRef<str>, S2: AsRef<str>>(dst: P, name: S1, value: S2, opts: &FsOptions) -> Result<(), io::Error> {
    let dst = dst.as_ref();    
    let value = value.as_ref();
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let name_cstr = CString::new(name.as_ref().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let value_cstr = CString::new(value.as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let code = unsafe {
        match opts.follow_symlinks {
            true => cstd_across_setxattr(
                dst_cstr.as_ptr(),
                name_cstr.as_ptr(),
                value_cstr.as_ptr() as *const libc::c_void,
                value.len(),
                0,
            ),
            false => cstd_across_lsetxattr(
                dst_cstr.as_ptr(),
                name_cstr.as_ptr(),
                value_cstr.as_ptr() as *const libc::c_void,
                value.len(),
                0,
            ),
        }
    };
    
    match code {
        0 => Ok(()),
        -1 => Er::setxattr.lasterr_unsupported_ok_if(opts.lossy_extended_attributes, opts),
        _ => Er::setxattr.err_unknown(&opts),
    }
}
