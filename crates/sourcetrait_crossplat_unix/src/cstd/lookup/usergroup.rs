use crate::*;

fn user_from_passwd(user_ptr: *mut libc::passwd) -> CstdResult<UserCstd> {
    let username;
    let uid;
    let primary_gid;
    unsafe {
        username = CStr::from_ptr((*user_ptr).pw_name);
        uid = (*user_ptr).pw_uid;
        primary_gid = (*user_ptr).pw_gid;
    }
    
    let username = TwoString::try_from_ffi_cstr(username)?;
    
    Ok(UserCstd {
        username,
        uid,
        primary_gid,
    })
}

pub fn cstd_lookup_user(uid: UID) -> CstdResult<Option<UserCstd>> {
    let user = unsafe {
        let user_ptr = ::libc::getpwuid(uid);
        if user_ptr.is_null() {
            return Ok(None);
        }
        
        user_from_passwd(user_ptr)?
    };
    
    Ok(Some(user))
}

pub fn cstd_lookup_username(username: TwoStr<'_>) -> CstdResult<Option<UserCstd>> {
    let user = unsafe {
        let username_cstr = CString::try_from(username)?;
        let user_ptr = ::libc::getpwnam(username_cstr.as_ptr());
        if user_ptr.is_null() {
            return Ok(None);
        }
        
        user_from_passwd(user_ptr)
    }?;
    
    Ok(Some(user))
}

fn new_group(group_ptr: *mut libc::group) -> CstdResult<UserGroupCstd> {
    let gid;
    let groupname;
    unsafe {
        gid = (*group_ptr).gr_gid;
        groupname = CStr::from_ptr((*group_ptr).gr_name);
    }
    
    let groupname = TwoString::try_from_ffi_cstr(groupname)?;
    
    Ok(UserGroupCstd {
        groupname,
        gid,
    })
}

pub fn cstd_lookup_group(gid: GID) -> CstdResult<Option<UserGroupCstd>> {
    let group = unsafe {
        let group_ptr = libc::getgrgid(gid);
        if group_ptr.is_null() {
            return Ok(None);
        }
        
        new_group(group_ptr)
    }?;
    
    Ok(Some(group))
}

pub fn cstd_lookup_groupname(groupname: TwoStr<'_>) -> CstdResult<Option<UserGroupCstd>> {
    let group = unsafe {
        let groupname_cstr = CString::try_from(groupname)?;
        let group_ptr = libc::getgrnam(groupname_cstr.as_ptr());
        if group_ptr.is_null() {
            return Ok(None);
        }
        
        new_group(group_ptr)
    }?;
    
    Ok(Some(group))
}

pub fn cstd_lookup_user_primary_group(uid: UID) -> CstdResult<UserGroupCstd> {
    let primary_gid = unsafe {
        let passwd_ptr = libc::getpwuid(uid);
        if passwd_ptr.is_null() {
            return CstdError::err_not_found(CstdEr::User);
        }
        
        (*passwd_ptr).pw_gid
    };
    
    cstd_lookup_group(primary_gid)?
        .ok_or_else(|| CstdError::not_found(CstdEr::UserGroup))
    
}

pub fn cstd_lookup_username_secondary_groups(username: TwoStr<'_>) -> CstdResult<HashSet<UserGroupCstd>> {
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
                    let group = new_group(group_ptr)?;
                    groups.insert(group);
                }
            }
        }
        libc::endgrent();
    }
    
    Ok(groups)
}

pub fn cstd_lookup_group_primary_users(gid: GID) -> CstdResult<HashSet<UserCstd>> {
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
        .map(|uid| cstd_lookup_user(uid))
        .collect::<CstdResult<HashSet<_>>>()?
        .into_iter()
        .filter_map(|opt| opt)
        .collect();
    
    Ok(users)
}

pub fn cstd_lookup_group_secondary_users(gid: GID) -> CstdResult<HashSet<UserCstd>> {
    let mut member_names = HashSet::new();
    
    unsafe {
        let group_ptr = libc::getgrgid(gid);
        if group_ptr.is_null() {
            return CstdError::err_not_found(CstdEr::UserGroup);
        }
        
        let members_ptr = (*group_ptr).gr_mem;
        if !members_ptr.is_null() {
            let mut i = 0;
            loop {
                let member_ptr = ptr::read(members_ptr.add(i));
                if member_ptr.is_null() {
                    break;
                }
                
                let member_name = TwoString::try_from_ffi_cstr(CStr::from_ptr(member_ptr))?;
                member_names.insert(member_name);
                i += 1;
            }
        }
    }
    
    let users = member_names.into_iter()
        .map(|username| cstd_lookup_username(username.as_two_str()))
        .collect::<CstdResult<HashSet<_>>>()?
        .into_iter()
        .filter_map(|opt| opt)
        .collect();
    
    Ok(users)
}

pub fn cstd_lookup_real_process_user() -> CstdResult<UserCstd> {
    let uid = unsafe { libc::getuid() };
    cstd_lookup_user(uid)?
        .ok_or_else(|| CstdError::not_found(CstdEr::User))
}

pub fn cstd_lookup_effective_process_user() -> CstdResult<UserCstd> {
    let uid = unsafe { libc::geteuid() };
    cstd_lookup_user(uid)?
        .ok_or_else(|| CstdError::not_found(CstdEr::User))
}

pub fn cstd_lookup_real_process_group() -> CstdResult<UserGroupCstd> {
    let gid = unsafe { libc::getgid() };
    cstd_lookup_group(gid)?
        .ok_or_else(|| CstdError::not_found(CstdEr::UserGroup))
}

pub fn cstd_lookup_effective_process_group() -> CstdResult<UserGroupCstd> {
    let gid = unsafe { libc::getegid() };
    cstd_lookup_group(gid)?
        .ok_or_else(|| CstdError::not_found(CstdEr::UserGroup))
}
