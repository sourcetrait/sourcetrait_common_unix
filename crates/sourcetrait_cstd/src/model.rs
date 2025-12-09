use crate::*;

#[derive(Debug, Clone)]
pub struct CStdUser {
    pub username: TwoString,
    pub uid: u32,
}

#[derive(Debug, Clone)]
pub struct UserGroupLibC {
    pub groupname: TwoString,
    pub gid: u32,
}