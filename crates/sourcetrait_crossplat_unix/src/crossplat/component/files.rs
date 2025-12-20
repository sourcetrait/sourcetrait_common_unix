use crate::*;

pub struct UnixFilesComponentLookup;
impl cross::FilesComponentLookup for UnixFilesComponentLookup {
    fn copy_preserved_with<P1, P2>(&self, src: P1, dst: P2, opts: &FsOptions) -> cross::BridgeResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>
    {
        crate::copy_preserved(src, dst, opts)?;
        Ok(())
    }
        
    fn own_capable_with<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: Capable<PrimaryUserGroupsCapable, GAID>,
        perms: BasicPermissionMode,
        opts: &FsOptions) -> BridgeResult<()>
    where
        UAID: AsAID,
        GAID: AsAID,
        P: AsRef<Path>,
    {
        let uid = user.as_aid().unix_id()?;
        let gid = group.ok()?.as_aid().unix_id()?;
        let mode = perms.to_unix_file_mode();
        crate::chown(&dst, Some(uid), Some(gid), opts)?;
        crate::chmod(dst, mode, opts)
    }
}
