use libc::{open, close, O_RDONLY};
use dylib_hook::bypass_hooks;
use std::ffi::CString;

#[test]
fn test_pread_block() {
    let path = CString::new("/etc/ld.so.preload").unwrap();

    // Use bypass_hooks to allow opening the file
    let fd = bypass_hooks(|| unsafe { open(path.as_ptr(), O_RDONLY) });
    assert!(fd >= 0, "Expected open to succeed for /etc/ld.so.preload within hook protection (errno: {:?})", errno::errno());

    // Attempt to read from the file descriptor outside of hook protection
    let mut buffer = [0u8; 128];
    let bytes_read = unsafe { library::pread(fd, buffer.as_mut_ptr() as *mut _, buffer.len(), 0) };
    assert_eq!(bytes_read, -1, "Expected pread to fail for /etc/ld.so.preload outside of hook protection");

    unsafe { close(fd) };
}

#[test]
fn test_pread_allow() {
    let path = CString::new("/etc/passwd").unwrap();
    let fd = unsafe { open(path.as_ptr(), O_RDONLY) };
    assert!(fd >= 0, "Expected open to succeed for /etc/passwd");

    let mut buffer = [0u8; 128];
    let bytes_read = unsafe { library::pread(fd, buffer.as_mut_ptr() as *mut _, buffer.len(), 0) };
    assert!(bytes_read > 0, "Expected pread to succeed for /etc/passwd");

    unsafe { close(fd) };
}
