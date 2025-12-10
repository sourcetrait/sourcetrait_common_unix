mod tests {
    use sourcetrait_crossplat_unix::*;
    
    #[test]
    fn test_current_user() {
        let effective_user = cstd_lookup_effective_process_user().unwrap();
        let user = cstd_lookup_user(effective_user.uid).unwrap().unwrap();
        assert_eq!(effective_user, user);
    }
}