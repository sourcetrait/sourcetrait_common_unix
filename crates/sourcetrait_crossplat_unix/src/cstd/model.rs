use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserCstd {
    pub username: TwoString,
    pub uid: UID,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserGroupCstd {
    pub groupname: TwoString,
    pub gid: GID,
}

