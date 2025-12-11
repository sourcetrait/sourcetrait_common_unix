use crate::*;

pub struct UnixPathsComponentLookup;
impl cross::PathsComponentLookup for UnixPathsComponentLookup {
    fn lookup_env_paths(&self) -> cross::BridgeResult<Vec<PathBuf>> {
        let paths = env::var(ENV_PATH)
            .map_err(|e| cross::BridgeError::env_var(ENV_PATH, e))?;
        let paths = paths.split(':')
            .into_iter()
            .map(|path| path.trim())
            .map(PathBuf::from)
            .collect();
        
        Ok(paths)
    }
}