mod lsetxattr;
mod setxattr;

pub use self::{
    setxattr::cstd_across_setxattr,
    lsetxattr::cstd_across_lsetxattr,
};