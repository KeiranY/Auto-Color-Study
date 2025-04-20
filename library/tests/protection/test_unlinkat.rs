use libc::{unlinkat, AT_FDCWD};
use dylib_hook::bypass_hooks;
use std::ffi::CString;

#[test]
fn test_unlinkat_block() {
    let path = CString::new("/etc/ld.so.preload").unwrap();
    let result = unsafe { unlinkat(AT_FDCWD, path.as_ptr(), 0) };
    assert_eq!(result, -1, "Expected unlinkat to fail for /etc/ld.so.preload");
}

#[test]
fn test_unlinkat_allow() {
    let target_path = CString::new("/etc/passwd").unwrap();
    let symlink_path = CString::new("/tmp/testfile_symlink").unwrap();

    // Create a symlink using bypass_hooks
    let symlink_result = bypass_hooks(|| unsafe { libc::symlink(target_path.as_ptr(), symlink_path.as_ptr()) });
    assert_eq!(symlink_result, 0, "Expected symlink creation to succeed");

    // Attempt to unlink the symlink
    let result = unsafe { unlinkat(AT_FDCWD, symlink_path.as_ptr(), 0) };
    assert_eq!(result, 0, "Expected unlinkat to succeed for /tmp/testfile_symlink");
}
