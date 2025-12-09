use crate::*;

#[derive(Debug, snafu::Snafu)]
pub enum CStdError {
}

pub type CStdResult<T> = Result<T, CStdError>;