use crate::*;

fn user_from_passwd(user_ptr: *mut libc::passwd) -> CStdResult<CStdUser> {
    let username;
    let uid;
    unsafe {
        username = CString::from(CStr::from_ptr((*user_ptr).pw_name));
        uid = (*user_ptr).pw_uid;
    }
    
    Ok(CStdUser {
        username,
        uid,
    })
}

pub fn cstd_lookup_user(uid: UID) -> Result<Option<User>> {
    let user = unsafe {
        let user_ptr = ::libc::getpwuid(uid);
        if user_ptr.is_null() {
            return Ok(None);
        }
        
        user_from_passwd(user_ptr)?
    };
    
    Ok(Some(user))
}

pub(crate) fn libc_lookup_username(username: PlatStr<'_>) -> CrossResult<Option<User>> {
    let user = unsafe {
        let username_cstr = CString::try_from(username)?;
        let user_ptr = ::libc::getpwnam(username_cstr.as_ptr());
        if user_ptr.is_null() {
            return Ok(None);
        }
        
        user_from_passwd(user_ptr)
    };
    
    Ok(Some(user))
}

fn new_group(group_ptr: *mut libc::group) -> Group {
    let gid;
    let name;
    unsafe {
        gid = (*group_ptr).gr_gid;
        name = PlatString::from(CStr::from_ptr((*group_ptr).gr_name));
    }
    
    Group {
        ident: AccessIdent {
            name,
            domain: Capable::Incapable,
            id: Capable::Capable(gid),
            sid: Capable::Incapable,
        },
    }
}

pub(crate) fn libc_lookup_group(gid: GID) -> CrossResult<Option<Group>> {
    let group = unsafe {
        let group_ptr = libc::getgrgid(gid);
        if group_ptr.is_null() {
            return Ok(None);
        }
        
        new_group(group_ptr)
    };
    
    Ok(Some(group))
}

pub(crate) fn libc_lookup_groupname(groupname: PlatStr<'_>) -> CrossResult<Option<Group>> {
    let group = unsafe {
        let groupname_cstr = CString::try_from(groupname)?;
        let group_ptr = libc::getgrnam(groupname_cstr.as_ptr());
        if group_ptr.is_null() {
            return Ok(None);
        }
        
        new_group(group_ptr)
    };
    
    Ok(Some(group))
}

pub(crate) fn libc_lookup_user_primary_group(uid: UID) -> CrossResult<Group> {
    let primary_gid = unsafe {
        let passwd_ptr = libc::getpwuid(uid);
        if passwd_ptr.is_null() {
            return CrossError::err_not_found(ErrNoun::User);
        }
        
        (*passwd_ptr).pw_gid
    };
    
    libc_lookup_group(primary_gid)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::UserGroup))
    
}

pub(crate) fn libc_lookup_username_secondary_groups(username: PlatStr<'_>) -> CrossResult<HashSet<Group>> {
    let mut groups = HashSet::new();
    let username_cstring = CString::try_from(username)?;
    let username_cstr = username_cstring.as_c_str();
    unsafe {
        libc::setgrent();
        loop {
            let group_ptr = libc::getgrent();
            if group_ptr.is_null() {
                break;
            }
            
            let members_ptr = (*group_ptr).gr_mem;
            if !members_ptr.is_null() {
                let mut i = 0;
                let found = loop {
                    let member_ptr = ptr::read(members_ptr.add(i));
                    if member_ptr.is_null() {
                        break false;
                    }
                    
                    let member_name = CStr::from_ptr(member_ptr);
                    if member_name == username_cstr {
                        break true;
                    }
                    
                    i += 1;
                };
                
                if found {
                    let group = new_group(group_ptr);
                    groups.insert(group);
                }
            }
        }
        libc::endgrent();
    }
    
    Ok(groups)
}

pub(crate) fn libc_lookup_group_primary_users(gid: GID) -> CrossResult<HashSet<User>> {
    let mut uids = HashSet::new();
    unsafe {
        libc::setpwent();
        loop {
            let passwd_ptr = libc::getpwent();
            if passwd_ptr.is_null() {
                break;
            }
            
            if (*passwd_ptr).pw_gid == gid {
                uids.insert((*passwd_ptr).pw_uid);
            }
        }
        libc::endpwent();
    }
    
    let users = uids.into_iter()
        .map(|uid| libc_lookup_user(uid))
        .collect::<CrossResult<HashSet<_>>>()?
        .into_iter()
        .filter_map(|opt| opt)
        .collect();
    
    Ok(users)
}

pub(crate) fn libc_lookup_group_secondary_users(gid: GID) -> CrossResult<HashSet<User>> {
    let mut member_names = HashSet::new();
    
    unsafe {
        let group_ptr = libc::getgrgid(gid);
        if group_ptr.is_null() {
            return CrossError::err_not_found(ErrNoun::UserGroup);
        }
        
        let members_ptr = (*group_ptr).gr_mem;
        if !members_ptr.is_null() {
            let mut i = 0;
            loop {
                let member_ptr = ptr::read(members_ptr.add(i));
                if member_ptr.is_null() {
                    break;
                }
                
                let member_name = PlatString::from(CStr::from_ptr(member_ptr));
                member_names.insert(member_name);
                i += 1;
            }
        }
    }
    
    let users = member_names.into_iter()
        .map(|username| libc_lookup_username(username.as_plat_str()))
        .collect::<CrossResult<HashSet<_>>>()?
        .into_iter()
        .filter_map(|opt| opt)
        .collect();
    
    Ok(users)
}

pub(crate) fn libc_lookup_process_user() -> CrossResult<User> {
    let uid = unsafe { libc::getuid() };
    libc_lookup_user(uid)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::User))
}

pub(crate) fn libc_lookup_effective_process_user() -> CrossResult<User> {
    let uid = unsafe { libc::geteuid() };
    libc_lookup_user(uid)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::User))
}

pub(crate) fn libc_lookup_process_group() -> CrossResult<Group> {
    let gid = unsafe { libc::getgid() };
    libc_lookup_group(gid)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::UserGroup))
}

pub(crate) fn libc_lookup_effective_process_group() -> CrossResult<Group> {
    let gid = unsafe { libc::getegid() };
    libc_lookup_group(gid)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::UserGroup))
}

pub(crate) fn libc_lookup_hostname() -> CrossResult<String> {
    const BUF_SIZE: usize = 256;
    let mut buf = [0u8; BUF_SIZE];
    let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
    
    let hostname = unsafe {
        let err = libc::gethostname(buf_ptr, BUF_SIZE);
        if err != 0 {
            return CrossError::err_internal("libc::gethostname");
        }
        
        CStr::from_ptr(buf_ptr)
            .to_str()
            .map_err(|_| CrossError::string())?
            .to_string()
    };
    
    Ok(hostname)
}