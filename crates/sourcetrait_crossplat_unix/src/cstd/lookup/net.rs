use crate::*;

pub fn cstd_lookup_hostname() -> CstdResult<String> {
    const BUF_SIZE: usize = 256;
    let mut buf = [0u8; BUF_SIZE];
    let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
    
    let hostname = unsafe {
        let err = libc::gethostname(buf_ptr, BUF_SIZE);
        if err != 0 {
            let err = io::Error::last_os_error();
            return CstdError::err_sys_call(err, CstdEr::Hostname);
        }
        
        CStr::from_ptr(buf_ptr)
            .to_str()
            .map_err(|_| CstdError::String)?
            .to_string()
    };
    
    Ok(hostname)
}