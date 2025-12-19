mod tests {
    use sourcetrait_crossplat_unix as unix;
    use sourcetrait_testing::prelude::*;
    #[allow(unused_imports)]
    use std::{
        fs,
        os::unix::fs::{
            PermissionsExt,
            MetadataExt,
        },
    };
    
    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
        .using_temp_dir()
    });
    
    const EXPECTED_TXT: &'static str = "expected.txt";
    const ACTUAL_TXT: &'static str = "actual.txt";
    const EXPECTED_TMP_TXT: &'static str = "expected.tmp.txt";
    const EXPECTED_TMP_SYM_TXT: &'static str = "expected.tmp.sym.txt";
    const ACTUAL_TMP_TXT: &'static str = "actual.tmp.txt";
    const EXPECTED_TEST_XATTR_NAME: &'static str = "user.test";
    const EXPECTED_TEST_XATTR_VALUE: &'static str = "testing";
    
    #[tested]
    fn test_file() {
        let test = testing::test!({
            .inherit_fixture_dir()
            .using_temp_dir()
        });
        
        // copy_preserved() a fixture file into temp and compare
        let expected_txt = test.fixture_dir().join(EXPECTED_TXT);
        let actual_txt = test.temp_dir().join(ACTUAL_TXT);
        let options = unix::FsOptions::default();
        unix::copy_preserved(&expected_txt, &actual_txt, &options).unwrap();
        assert!(actual_txt.is_file());
        assert_eq!(fs::read_to_string(&expected_txt).unwrap(), fs::read_to_string(&actual_txt).unwrap());
        let expected_meta = fs::metadata(&expected_txt).unwrap();
        let actual_meta = fs::metadata(&actual_txt).unwrap();
        assert_eq!(expected_meta.permissions(), actual_meta.permissions());
        assert_eq!(expected_meta.permissions().mode(), actual_meta.permissions().mode());
        assert_eq!(expected_meta.modified().unwrap(), actual_meta.modified().unwrap());
        
        // copy_preserved() the fixture file into tmp, set an extended attribute on it, copy_preserved() it to another
        // tmp file, check to see if it the attribute exists
        let expected_tmp_txt = test.temp_dir().join(EXPECTED_TMP_TXT);
        let actual_tmp_txt = test.temp_dir().join(ACTUAL_TMP_TXT);
        unix::copy_preserved(&expected_txt, &expected_tmp_txt, &options).unwrap();
        unix::set_extended_attribute(&expected_tmp_txt, EXPECTED_TEST_XATTR_NAME, EXPECTED_TEST_XATTR_VALUE, &options).unwrap();
        unix::copy_preserved(&expected_tmp_txt, &actual_tmp_txt, &options).unwrap();
        let actual = unix::get_extended_attribute(&actual_tmp_txt, EXPECTED_TEST_XATTR_NAME, &options).unwrap();
        assert_eq!(Some(String::from(EXPECTED_TEST_XATTR_VALUE)), actual);
    }
    
    #[tested]
    fn test_symlink_follow() {
        let test = testing::test!({
            .inherit_fixture_dir()
            .using_temp_dir()
        });
        
        // copy_preserved() the fixture file into tmp, set an extended attribute on it,
        // symlink it, copy_preserved() the symlink, check
        let expected_txt = test.fixture_dir().join(EXPECTED_TXT);
        let expected_tmp_txt = test.temp_dir().join(EXPECTED_TMP_TXT);
        let expected_tmp_sym_txt = test.temp_dir().join(EXPECTED_TMP_SYM_TXT);
        let actual_txt = test.temp_dir().join(ACTUAL_TXT);
        let options = unix::FsOptions::default();
        unix::copy_preserved(&expected_txt, &expected_tmp_txt, &options).unwrap();
        unix::set_extended_attribute(&expected_tmp_txt, EXPECTED_TEST_XATTR_NAME, EXPECTED_TEST_XATTR_VALUE, &options).unwrap();
        std::os::unix::fs::symlink(&expected_tmp_txt, &expected_tmp_sym_txt).unwrap();
        assert!(expected_tmp_sym_txt.is_symlink());
        unix::copy_preserved(&expected_tmp_sym_txt, &actual_txt, &options).unwrap();
        assert!(!actual_txt.is_symlink());
        assert_eq!(fs::read_to_string(&expected_txt).unwrap(), fs::read_to_string(&actual_txt).unwrap());
        let expected_meta = fs::metadata(&expected_tmp_txt).unwrap();
        let actual_meta = fs::metadata(&actual_txt).unwrap();
        assert_eq!(expected_meta.permissions(), actual_meta.permissions());
        assert_eq!(expected_meta.permissions().mode(), actual_meta.permissions().mode());
        assert_eq!(expected_meta.modified().unwrap(), actual_meta.modified().unwrap());
        let actual = unix::get_extended_attribute(&actual_txt, EXPECTED_TEST_XATTR_NAME, &options).unwrap();
        assert_eq!(Some(String::from(EXPECTED_TEST_XATTR_VALUE)), actual);
    }
    
  #[tested]
    fn test_symlink_nofollow() {
        let test = testing::test!({
            .inherit_fixture_dir()
            .using_temp_dir()
        });
        
        // copy_preserved() the fixture file into tmp, set an extended attribute on it,
        // symlink it, copy_preserved() the symlink, check
        let expected_txt = test.fixture_dir().join(EXPECTED_TXT);
        let expected_tmp_txt = test.temp_dir().join(EXPECTED_TMP_TXT);
        let expected_tmp_sym_txt = test.temp_dir().join(EXPECTED_TMP_SYM_TXT);
        let actual_txt = test.temp_dir().join(ACTUAL_TXT);
        let options_follow = unix::FsOptions::default();
        let options = unix::FsOptions::default_nofollow();
        unix::copy_preserved(&expected_txt, &expected_tmp_txt, &options_follow).unwrap();
        unix::set_extended_attribute(&expected_tmp_txt, EXPECTED_TEST_XATTR_NAME, EXPECTED_TEST_XATTR_VALUE, &options_follow).unwrap();
        std::os::unix::fs::symlink(&expected_tmp_txt, &expected_tmp_sym_txt).unwrap();
        assert!(expected_tmp_sym_txt.is_symlink());
        unix::copy_preserved(&expected_tmp_sym_txt, &actual_txt, &options).unwrap();
        assert!(actual_txt.is_symlink());
        assert_eq!(fs::read_to_string(&expected_txt).unwrap(), fs::read_to_string(&actual_txt).unwrap());
        let expected_meta = fs::metadata(&expected_tmp_txt).unwrap();
        let expected_sym_meta = fs::symlink_metadata(&expected_tmp_sym_txt).unwrap();
        let actual_meta = fs::symlink_metadata(&actual_txt).unwrap();
        assert_eq!(expected_sym_meta.permissions(), actual_meta.permissions());
        assert_eq!(expected_sym_meta.permissions().mode(), actual_meta.permissions().mode());
        assert_ne!(expected_meta.permissions().mode(), actual_meta.permissions().mode());
        assert_eq!(expected_sym_meta.modified().unwrap(), actual_meta.modified().unwrap());
        assert_ne!(expected_meta.modified().unwrap(), actual_meta.modified().unwrap());
        let actual = unix::get_extended_attribute(&actual_txt, EXPECTED_TEST_XATTR_NAME, &options).unwrap();
        assert_eq!(None, actual);
        let actual = unix::get_extended_attribute(&actual_txt, EXPECTED_TEST_XATTR_NAME, &options_follow).unwrap();
        assert_eq!(Some(String::from(EXPECTED_TEST_XATTR_VALUE)), actual);
    }
}
