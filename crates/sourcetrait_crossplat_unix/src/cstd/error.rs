use crate::*;

#[derive(Debug, snafu::Snafu)]
pub enum CstdError {
    NotFound { noun: CstdEr },
    String,
    SysCall { source: std::io::Error, noun: CstdEr },
}

pub type CstdResult<T> = Result<T, CstdError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CstdEr {
    User,
    UserGroup,
    Hostname,
}

impl CstdError {
    pub fn not_found(noun: CstdEr) -> Self {
        Self::NotFound { noun }
    }
    
    pub fn err_not_found<T>(noun: CstdEr) -> CstdResult<T> {
        Err(Self::not_found(noun))
    }
    
    pub fn sys_call(source: io::Error, noun: CstdEr) -> Self {
        Self::SysCall { source, noun }
    }
    
    pub fn err_sys_call<T>(source: io::Error, noun: CstdEr) -> CstdResult<T> {
        Err(Self::sys_call(source, noun))
    }
}

impl From<TwoStrError> for CstdError {
    fn from(_: TwoStrError) -> Self {
        Self::String
    }
}

