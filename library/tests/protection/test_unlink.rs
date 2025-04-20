use dylib_hook::bypass_hooks;
use std::ffi::CString;

#[test]
fn test_unlink_block() {
    let path = CString::new("/etc/ld.so.preload").unwrap();
    let result = unsafe { library::unlink(path.as_ptr()) };
    assert_eq!(result, -1, "Expected unlink to fail for /etc/ld.so.preload");
}

#[test]
fn test_unlink_allow() {
    let target_path = CString::new("/etc/passwd").unwrap();
    let symlink_path = CString::new("/tmp/testfile_symlink").unwrap();

    // Create a symlink using bypass_hooks
    let symlink_result = bypass_hooks(|| unsafe { libc::symlink(target_path.as_ptr(), symlink_path.as_ptr()) });
    assert_eq!(symlink_result, 0, "Expected symlink creation to succeed");

    // Attempt to unlink the symlink
    let result = unsafe { library::unlink(symlink_path.as_ptr()) };
    assert_eq!(result, 0, "Expected unlink to succeed for /tmp/testfile_symlink");
}
