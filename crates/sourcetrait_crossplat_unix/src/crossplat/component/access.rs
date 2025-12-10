use crate::*;

pub(crate) struct UnixAccessComponentLookup;
impl AccessComponentLookup for UnixAccessComponentLookup {
    const LOOKUP: AccessLookup = AccessLookup {
        lookup_user_fn,
        lookup_group_fn,
        lookup_user_groups_fn,
        lookup_group_users_fn,
        lookup_current_user_fn,
    };
}

fn lookup_user_fn(query: Access<'_>) -> CrossBridgeResult<Option<User>> {
    Ok(match query {
        Access::Name(name) => cstd_lookup_username(name),
        Access::ID(id) => cstd_lookup_user(id),
        Access::QualifiedName(_, _) => CrossBridgeError::err_incapable(Capability::QualifiedAccessNames),
        Access::SID(_) => CrossBridgeError::err_incapable(Capability::WindowsSIDs),
    })
}

fn lookup_group_fn(query: Access) -> CrossBridgeResult<Option<Group>> {
    match query {
        Access::Name(name) => cstd_lookup_groupname(name),
        Access::ID(id) => cstd_lookup_group(id),
        Access::QualifiedName(_, _) => CrossBridgeError::err_incapable(Capability::QualifiedAccessNames),
        Access::SID(_) => CrossBridgeError::err_incapable(Capability::WindowsSIDs),
    }
}

fn lookup_current_user_fn() -> CrossBridgeResult<User> {
    cstd_lookup_process_user()
}

fn lookup_cstd_effective_process_user() -> CrossBridgeResult<User> {
    cstd_lookup_process_user()
}

fn lookup_cstd_process_group() -> CrossBridgeResult<Group> {
    cstd_lookup_process_group()
}

fn lookup_cstd_effective_process_group() -> CrossBridgeResult<Group> {
    cstd_lookup_process_group()
}

fn lookup_user_groups_fn(user: &User) -> CrossBridgeResult<(Vec<Group>, Capable<AccessKey>)> {
    let mut groups = cstd_lookup_username_secondary_groups(user.username())?;
    let primary_group = cstd_lookup_user_primary_group(user.uid())?;
    let primary_group_key = primary_group.key_identifier();
    groups.insert(primary_group);
    let groups = groups.into_iter().collect();
    Ok((groups, Capable::Capable(primary_group_key)))
}

fn lookup_group_users_fn(group: &Group) -> CrossBridgeResult<Vec<User>> {
    let gid = group.gid();
    let primary_users = cstd_lookup_group_primary_users(gid)?;
    let secondary_users = cstd_lookup_group_secondary_users(gid)?;
    
    let users = primary_users.into_iter()
        .chain(secondary_users.into_iter())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    
    Ok(users)
}
