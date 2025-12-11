use crate::*;

pub fn lookup_hostname() -> cross::BridgeResult<String> {
    cstd_lookup_hostname().map_err(cross::BridgeError::from)
}

