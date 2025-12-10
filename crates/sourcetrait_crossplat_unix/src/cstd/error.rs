use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, snafu::Snafu)]
pub enum CstdError {
    NotFound { noun: CstdEr },
    String,
    SysCall { noun: CstdEr },
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
    
    pub fn sys_call(noun: CstdEr) -> Self {
        Self::SysCall { noun }
    }
    
    pub fn err_sys_call<T>(noun: CstdEr) -> CstdResult<T> {
        Err(Self::sys_call(noun))
    }
}

impl From<TwoStrError> for CstdError {
    fn from(_: TwoStrError) -> Self {
        Self::String
    }
}