use crate::*;

impl From<CstdError> for cross::BridgeError {
    fn from(e: CstdError) -> Self {
        match e {
            CstdError::NotFound { noun } => Self::NotFound { noun: noun.into() },
            CstdError::String => Self::String,
            CstdError::SysCall { noun } => Self::SysCall { noun: noun.into() },
        }
    }
}

impl From<CstdEr> for cross::BridgeErr {
    fn from(er: CstdEr) -> Self {
        match er {
            CstdEr::User => Self::User,
            CstdEr::UserGroup => Self::UserGroup,
            CstdEr::Hostname => Self::Hostname,
        }
    }
}