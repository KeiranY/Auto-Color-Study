use libc::closedir;
use std::ffi::CString;

#[ignore]
#[test]
fn test_opendir_block() {
    // let path = CString::new("/etc/ld.so.preload").unwrap();
    // let dir = unsafe { library::opendir(path.as_ptr()) };
    // assert!(dir.is_null(), "Expected opendir to fail for /etc/ld.so.preload");
}

#[ignore]
#[test]
fn test_opendir_allow() {
    // let path = CString::new("/etc").unwrap();
    // let dir = unsafe { library::opendir(path.as_ptr()) };
    // assert!(!dir.is_null(), "Expected opendir to succeed for /etc");

    // unsafe { closedir(dir) };
}
