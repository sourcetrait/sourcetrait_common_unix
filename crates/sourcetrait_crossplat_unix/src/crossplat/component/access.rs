use crate::*;

pub struct UnixAccessComponentLookup;
impl cross::AccessComponentLookup for UnixAccessComponentLookup {
    const LOOKUP: cross::AccessLookup = cross::AccessLookup {
        lookup_user_fn,
        lookup_group_fn,
        lookup_user_groups_fn,
        lookup_group_users_fn,
        lookup_effective_process_user_fn,
    };
}

fn lookup_user_fn(query: cross::Access<'_>) -> cross::BridgeResult<Option<cross::User>> {
    let user = match query {
        cross::Access::Name(name) => cstd_lookup_username(name)?,
        cross::Access::ID(id) => cstd_lookup_user(id)?,
        cross::Access::QualifiedName(_, _) => cross::BridgeError::err_incapable(cross::Capability::QualifiedAccessNames)?,
        cross::Access::SID(_) => cross::BridgeError::err_incapable(cross::Capability::WindowsSIDs)?,
    };
    
    let user = user.map(cross::User::from);
    Ok(user)
}

fn lookup_group_fn(query: cross::Access) -> cross::BridgeResult<Option<cross::UserGroup>> {
    let group = match query {
        cross::Access::Name(name) => cstd_lookup_groupname(name)?,
        cross::Access::ID(id) => cstd_lookup_group(id)?,
        cross::Access::QualifiedName(_, _) => cross::BridgeError::err_incapable(cross::Capability::QualifiedAccessNames)?,
        cross::Access::SID(_) => cross::BridgeError::err_incapable(cross::Capability::WindowsSIDs)?,
    };
    
    let group = group.map(cross::UserGroup::from);
    Ok(group)
}

fn lookup_effective_process_user_fn() -> cross::BridgeResult<cross::User> {
    cstd_lookup_effective_process_user().map(cross::User::from)
        .map_err(|e| e.into())
}

fn _lookup_cstd_real_process_user() -> cross::BridgeResult<cross::User> {
    cstd_lookup_real_process_user().map(cross::User::from)
        .map_err(|e| e.into())
}

fn _lookup_cstd_effective_process_group() -> cross::BridgeResult<cross::UserGroup> {
    cstd_lookup_effective_process_group().map(cross::UserGroup::from)
        .map_err(|e| e.into())
}

fn _lookup_cstd_real_process_group() -> cross::BridgeResult<cross::UserGroup> {
    cstd_lookup_real_process_group().map(cross::UserGroup::from)
        .map_err(|e| e.into())
}

fn lookup_user_groups_fn(user: &cross::User) -> cross::BridgeResult<(Vec<cross::UserGroup>, cross::Capable<cross::AccessKey>)> {
    let mut groups = cstd_lookup_username_secondary_groups(user.username())?
        .into_iter()
        .map(cross::UserGroup::from)
        .collect::<HashSet<_>>();
    
    let primary_group = cstd_lookup_user_primary_group(user.uid())
        .map(cross::UserGroup::from)?;
    
    let primary_group_key = primary_group.key_identifier();
    groups.insert(primary_group);
    
    let groups = groups.into_iter().map(cross::UserGroup::from).collect();
    Ok((groups, cross::Capable::Capable(primary_group_key)))
}

fn lookup_group_users_fn(group: &cross::UserGroup) -> cross::BridgeResult<Vec<cross::User>> {
    let gid = group.gid();
    let primary_users = cstd_lookup_group_primary_users(gid)?;
    let secondary_users = cstd_lookup_group_secondary_users(gid)?;
    
    let users = primary_users.into_iter()
        .chain(secondary_users.into_iter())
        .map(cross::User::from)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    
    Ok(users)
}
