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
pub(crate) mod unix_fs {
    pub(crate) mod er;
    pub(crate) mod options;
    pub(crate) mod copy_preserved;
    pub(crate) mod extended_attributes;
    
    pub(crate) mod prelude {
        pub(crate) use super::er::Er;
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
    },
    unix_fs::{
        options::FsOptions,
        copy_preserved::{CopyError, copy_preserved},
        extended_attributes::{get_extended_attribute, set_extended_attribute},
    },
};

#[allow(unused_imports)]
pub(crate) use std::{
    collections::HashSet,
    env,
    ffi::{CStr, CString},
    io,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::MetadataExt,
    },
    path::{Path, PathBuf},
    ptr,
    process::Command,
};

pub(crate) use sourcetrait_twostr::*;
pub(crate) use sourcetrait_crossplat_bridge::{
    self as cross,
    prelude::driver::*
};
pub(crate) use const_format::concatcp;