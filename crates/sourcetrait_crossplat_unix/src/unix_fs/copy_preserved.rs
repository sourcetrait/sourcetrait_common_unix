use crate::*;

/// Copies a file while preserving all metadata:
/// - Created and modified times
/// - Extended attributes
/// - 
pub fn copy_preserved<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2) -> Result<(), io::Error> {
    fs::copy(&src, &dst)?;
    
    let src = src.as_ref();
    let dst = dst.as_ref();
    let metadata = src.metadata()?;
    let src_cstr = CString::new(src.as_os_str().as_bytes())?;
    let dst_cstr = CString::new(dst.as_os_str().as_bytes())?;
    
    unsafe {
        // ownership
        libc::chown(
            dst_cstr.as_ptr(),
            metadata.uid(),
            metadata.gid(),
        );
        
        // permissons
        libc::chmod(dst_cstr.as_ptr(), metadata.mode());
        
        let times = [
            libc::timespec {
                tv_sec: metadata.atime(),
                tv_nsec: metadata.atime_nsec(),
            },
            libc::timespec {
                tv_sec: metadata.mtime(),
                tv_nsec: metadata.mtime_nsec(),
            },
        ];
        
        // created and modified times
        libc::utimensat(
            libc::AT_FDCWD,
            dst_cstr.as_ptr(),
            times.as_ptr(),
            0,
        );
    }
    
    // extended attributes
    copy_xattrs(&src_cstr, &dst_cstr)?;
    
    Ok(())
}

fn copy_xattrs(src: &CStr, dst: &CStr) -> io::Result<()> {
    unsafe {
        let size = libc::llistxattr(
            src.as_ptr(),
            ptr::null_mut(),
            0,
        );
        
        if size < 0 {
            let err = io::Error::last_os_error();
            return match err.raw_os_error().unwrap_or(i32::MIN) {
                libc::ENOTSUP => Ok(()),
                _ => Err(err),
            };
        } else if size < 1 {
            return Ok(());
        }
        
        let mut list = vec![0u8; size as usize];
        let code = libc::llistxattr(
            src.as_ptr(),
            list.as_mut_ptr() as *mut i8,
            size as usize,
        );
        
        if code < 0 {
            let err = io::Error::last_os_error();
            return match err.raw_os_error().unwrap_or(i32::MIN) {
                libc::ENOTSUP => Ok(()),
                _ => Err(err),
            };
        } else if code < 1 {
            return Ok(());
        }
        
        let mut pos = 0;
        while pos < list.len() {
            let name_start = pos;
            while pos < list.len() && list[pos] != 0 {
                pos += 1;
            }
            
            if pos <= name_start {
                break;
            }
            
            let name = CStr::from_bytes_with_nul(&list[name_start..=pos])
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid xattr name"))?;
            
            let value_size = libc::lgetxattr(
                src.as_ptr(),
                name.as_ptr(),
                ptr::null_mut(),
                0,
            );
            
            if value_size > 0 {
                let mut value = vec![0u8; value_size as usize];
                libc::getxattr(
                    src.as_ptr(),
                    name.as_ptr(),
                    value.as_mut_ptr() as *mut libc::c_void,
                    value_size as usize,
                );
                
                libc::setxattr(
                    dst.as_ptr(),
                    name.as_ptr(),
                    value.as_ptr() as *const libc::c_void,
                    value.len(),
                    0,
                );
            }
            
            pos += 1;
        }
    }
    
    Ok(())
}
