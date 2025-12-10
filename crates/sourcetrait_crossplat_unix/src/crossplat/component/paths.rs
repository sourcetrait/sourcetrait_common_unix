use crate::*;

pub(crate) struct UnixPathsComponentLookup;
impl PathsComponentLookup for UnixPathsComponentLookup {
    fn lookup_env_paths(&self) -> CrossLookupResult<Vec<PathBuf>> {
        let paths = env::var(ENV_PATH)
            .map_err(|e| CrossLookupError::env_var(ENV_PATH, e))?;
        let paths = paths.split(':')
            .into_iter()
            .map(|path| path.trim())
            .map(PathBuf::from)
            .collect();
        
        Ok(paths)
    }
}