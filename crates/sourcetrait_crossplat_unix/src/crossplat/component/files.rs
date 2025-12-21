use crate::*;

pub struct UnixFilesComponentLookup;
impl cross::FilesComponentLookup for UnixFilesComponentLookup {
    fn copy_preserved_with<P1, P2>(&self, src: P1, dst: P2, opts: impl AsRef<FsOptions>) -> cross::BridgeResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>
    {
        crate::copy_preserved(src, dst, opts.as_ref())?;
        Ok(())
    }
        
    fn own_capable_with<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
        opts: impl AsRef<FsOptions>
    ) -> BridgeResult<()>
    where
        UAID: AsAID,
        GAID: AsAID,
        P: AsRef<Path>,
    {
        let uid = user.as_aid().unix_id()?;
        let gid = group.as_ref().ok()?.as_aid().unix_id()?;
        let mode = perms.as_ref().to_unix_file_mode();
        crate::chown(&dst, Some(uid), Some(gid), opts.as_ref())?;
        crate::chmod(dst, mode, opts.as_ref())
    }
}
