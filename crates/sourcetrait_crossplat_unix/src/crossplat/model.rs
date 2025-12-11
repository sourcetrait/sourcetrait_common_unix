use crate::*;

impl From<UserCstd> for cross::User {
    fn from(u: UserCstd) -> Self {
        cross::User {
            ident: cross::AccessIdent {
                name: u.username,
                domain: cross::Capable::Incapable,
                id: cross::Capable::Capable(u.uid),
                sid: cross::Capable::Incapable,
            },
        }
    }
}

impl From<UserGroupCstd> for cross::UserGroup {
    fn from(g: UserGroupCstd) -> Self {
        cross::UserGroup {
            ident: cross::AccessIdent {
                name: g.groupname,
                domain: cross::Capable::Incapable,
                id: cross::Capable::Capable(g.gid),
                sid: cross::Capable::Incapable,
            },
        }
    }
}