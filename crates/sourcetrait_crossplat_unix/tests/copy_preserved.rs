mod tests {
    use sourcetrait_crossplat_unix as unix;
    use sourcetrait_testing::prelude::*;
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
    const EXPECTED_TMP_TXT: &'static str = "expected.tmp.txt";
    const ACTUAL_TMP_TXT: &'static str = "actual.tmp.txt";
    
    #[tested]
    fn test_copy_fixture() {
        let test = testing::test!({
            .inherit_fixture_dir()
            .using_temp_dir()
        });
        
        // copy_preserved() a fixture file into temp and compare
        let expected_txt = test.fixture_dir().join(EXPECTED_TXT);
        let actual_txt = test.temp_dir().join(EXPECTED_TXT);
        let options = unix::FsOptions::default();
        unix::copy_preserved(&expected_txt, &actual_txt, &options).unwrap();
        assert!(actual_txt.is_file());
        assert_eq!(fs::read_to_string(&expected_txt).unwrap(), fs::read_to_string(&actual_txt).unwrap());
        let expected_meta = fs::metadata(&actual_txt).unwrap();
        let actual_meta = fs::metadata(&actual_txt).unwrap();
        assert_eq!(expected_meta.permissions(), actual_meta.permissions());
        assert_eq!(expected_meta.permissions().mode(), actual_meta.permissions().mode());
        assert_eq!(expected_meta.created().unwrap(), actual_meta.created().unwrap());
        assert_eq!(expected_meta.modified().unwrap(), actual_meta.modified().unwrap());
        
        // copy_preserved() the fixture file into tmp, set an extended attribute on it, copy_preserved() it to another
        // tmp file, check to see if it the attribute exists
        let expected_tmp_txt = test.temp_dir().join(EXPECTED_TMP_TXT);
        let actual_tmp_txt = test.temp_dir().join(ACTUAL_TMP_TXT);
        const EXPECTED_TEST_XATTR_NAME: &'static str = "user.test";
        const EXPECTED_TEST_XATTR_VALUE: &'static str = "testing";
        unix::copy_preserved(&expected_txt, &expected_tmp_txt, &options).unwrap();
        unix::set_extended_attribute(&expected_tmp_txt, EXPECTED_TEST_XATTR_NAME, EXPECTED_TEST_XATTR_VALUE, &options).unwrap();
        unix::copy_preserved(&expected_tmp_txt, &actual_tmp_txt, &options).unwrap();
        let actual = unix::get_extended_attribute(&actual_tmp_txt, EXPECTED_TEST_XATTR_NAME, &options).unwrap();
        assert_eq!(Some(String::from(EXPECTED_TEST_XATTR_VALUE)), actual);
    }
}
