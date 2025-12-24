pub const fn across_e_no_data(e: libc::c_int) -> bool {
    match e {
        libc::ENODATA => true,
        #[cfg(target_os = "macos")]
        libc::ENOATTR => true,
        _ => false,
    }
}