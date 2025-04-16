extern crate libc;
extern crate errno;
use libc::{c_char, c_int, ENOENT};
use errno::set_errno;
use errno::Errno;
use std::{ffi::CStr, fs};

pub fn handle_rename(oldpath: *const c_char, newpath: *const c_char) -> Option<c_int> {
    let old_path = unsafe { CStr::from_ptr(oldpath) };
    let new_path = unsafe { CStr::from_ptr(newpath) };

    eprintln!("[rename] old: {:?}, new: {:?}", old_path, new_path);

    if old_path.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[rename] Prevented renaming of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }
    None
}

pub fn handle_stat(path: *const c_char) -> Option<c_int> {
    let path_str = unsafe { CStr::from_ptr(path) };

    eprintln!("[stat] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[stat] Hiding /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }
    None
}

pub fn handle_access(path: *const c_char, mode: c_int) -> Option<c_int> {
    let path_str = unsafe { CStr::from_ptr(path) };

    eprintln!("[access] path: {:?}, mode: {}", path_str, mode);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[access] Prevented access to /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }
    None
}

pub fn handle_realpath(path: *const c_char, _resolved_path: *mut c_char) -> Option<*mut c_char> {
    let path_str = unsafe { CStr::from_ptr(path) };

    eprintln!("[realpath] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[realpath] Prevented resolution of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(std::ptr::null_mut());
    }
    None
}

pub fn handle_open(cpath: *const c_char, _oflag: c_int) -> Option<c_int> {
    let path = unsafe { CStr::from_ptr(cpath) };

    eprintln!("[open] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[open] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }
    None
}

pub fn handle_openat(dirfd: c_int, cpath: *const c_char, _oflag: c_int) -> Option<c_int> {
    let resolved_path = crate::resolve_fd_path(dirfd, cpath)?;
    eprintln!("[openat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[openat] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }
    None
}

pub fn handle_fopen(cpath: *const c_char, _mode: *const c_char) -> Option<*mut libc::FILE> {
    let path = unsafe { CStr::from_ptr(cpath) };
    eprintln!("[fopen] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        eprintln!("[fopen] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(std::ptr::null_mut());
    }
    None
}

pub fn handle_read(fd: c_int, _buf: *mut libc::c_void, count: usize) -> Option<isize> {
    eprintln!("[read] fd: {}, count: {}", fd, count);

    let fd_path = format!("/proc/self/fd/{}", fd);
    let link_path = fs::read_link(&fd_path).ok()?;
    if  link_path.as_os_str() == "/etc/ld.so.preload" {
        eprintln!("[read] Prevented reading from /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return Some(-1);
    }

    None
}
