#[cfg(feature = "crossplat")]
pub(crate) mod crossplat {
    pub(crate) mod component {
        pub(crate) mod access;
        pub(crate) mod net;
        pub(crate) mod paths;
    }
    pub(crate) mod consts;
    pub(crate) mod error;
    pub(crate) mod model;
}
pub(crate) mod cstd {
    pub(crate) mod error;
    pub(crate) mod model;
    pub(crate) mod lookup {
        pub(crate) mod usergroup;
        pub(crate) mod net;
    }
}

pub use crate::{
    crossplat::{
        component::{
            access::*,
            net::*,
            paths::*,
        },
        consts::*,
    },
    cstd::{
        error::*,
        lookup::{
            usergroup::*,
            net::*,
        },
        model::*,
    }
};

pub(crate) use std::{
    collections::HashSet,
    env,
    ffi::{CStr, CString},
    path::{PathBuf},
    ptr,
};

pub(crate) use sourcetrait_twostr::*;
pub(crate) use sourcetrait_crossplat_bridge::{
    self as cross,
    prelude::driver::*
};