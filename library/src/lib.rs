extern crate libc;
extern crate ctor;
extern crate log;
extern crate colog;

use libc::{
    c_char, c_int, PATH_MAX
};
use std::{
    ffi::CStr, path::{Path, PathBuf}, env
};
use std::cell::RefCell;

// Thread-local flag to track internal calls
thread_local! {
    static IN_HOOK: RefCell<bool> = RefCell::new(false);
}

fn with_hook_protection<F, G, R>(f: F, f2: G) -> R
where
    F: FnOnce() -> Option<R>,
    G: FnOnce() -> R,
{
    IN_HOOK.with(|flag| {
        if *flag.borrow() {
            // If already in a hook, bypass and execute the real function
            return f2();
        }
        *flag.borrow_mut() = true; 
        let result = f().unwrap_or_else(f2); 
        *flag.borrow_mut() = false;
        result
    })
}

pub fn resolve_fd_path(dirfd: c_int, cpath: *const c_char) -> Option<PathBuf> {
    let path = unsafe { CStr::from_ptr(cpath) };

    if path.to_bytes().starts_with(b"/") {
        return Some(PathBuf::from(path.to_string_lossy().to_string()));
    }

    if dirfd == libc::AT_FDCWD {
        return std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(path.to_string_lossy().as_ref()));
    }

    let fd_path = PathBuf::from(format!("/proc/self/fd/{}", dirfd));
    let mut buf = vec![0; PATH_MAX as usize];
    let len = unsafe {
        libc::readlink(
            fd_path.to_string_lossy().as_ptr() as *const c_char,
            buf.as_mut_ptr() as *mut c_char,
            PATH_MAX as usize,
        )
    };

    if len == -1 {
        log::error!("[resolve_fd_path] Failed to resolve dirfd");
        return None;
    }

    let dir_path_str = unsafe { CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string() };
    let dir_path = Path::new(&dir_path_str);
    Some(dir_path.join(path.to_string_lossy().as_ref()))
}

#[ctor::ctor]
fn init() {
    if let Ok(log_level) = env::var("AUTO_COLOR_LOG") {
        colog::default_builder()
            .filter_level(match log_level.to_lowercase().as_str() {
                "error" => log::LevelFilter::Error,
                "warn" => log::LevelFilter::Warn,
                "info" => log::LevelFilter::Info,
                "debug" => log::LevelFilter::Debug,
                "trace" => log::LevelFilter::Trace,
                _ => log::LevelFilter::Off,    
            })
            .init();
    }
    log::info!("[library] Initialization function called");
}

mod hook_tcp;
mod hook_protection;

redhook::hook! {
    unsafe fn open(cpath: *const c_char, oflag: c_int) -> c_int => protect_open {
        with_hook_protection(
            || hook_protection::handle_open(cpath, oflag)
                .or_else(|| hook_tcp::handle_open(cpath, oflag)),
            || unsafe { redhook::real!(open)(cpath, oflag) }
        )
    }
}

redhook::hook! {
    unsafe fn open64(cpath: *const c_char, oflag: c_int) -> c_int => protect_open64 {
        with_hook_protection(
            || hook_protection::handle_open(cpath, oflag)
                .or_else(|| hook_tcp::handle_open(cpath, oflag)),
            || unsafe { redhook::real!(open64)(cpath, oflag) }
        )
    }
}

redhook::hook! {
    unsafe fn openat(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int => protect_openat {
        with_hook_protection(
            || hook_protection::handle_openat(dirfd, cpath, oflag)
                .or_else(|| hook_tcp::handle_openat(dirfd, cpath, oflag)),
            || unsafe { redhook::real!(openat)(dirfd, cpath, oflag) }
        )
    }
}

redhook::hook! {
    unsafe fn openat64(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int => protect_openat64 {
        with_hook_protection(
            || hook_protection::handle_openat(dirfd, cpath, oflag)
                .or_else(|| hook_tcp::handle_openat(dirfd, cpath, oflag)),
            || unsafe { redhook::real!(openat64)(dirfd, cpath, oflag) }
        )
    }
}

redhook::hook! {
    unsafe fn fopen(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE => protect_fopen {
        with_hook_protection(
            || hook_protection::handle_fopen(cpath, mode)
                .or_else(|| hook_tcp::handle_fopen(cpath, mode)),
            || unsafe { redhook::real!(fopen)(cpath, mode) }
        )
    }
}

redhook::hook! {
    unsafe fn fopen64(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE => protect_fopen64 {
        with_hook_protection(
            || hook_protection::handle_fopen(cpath, mode)
                .or_else(|| hook_tcp::handle_fopen(cpath, mode)),
            || unsafe { redhook::real!(fopen64)(cpath, mode) }
        )
    }
}

redhook::hook! {
    unsafe fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int => protect_rename {
        with_hook_protection(
            || hook_protection::handle_rename(oldpath, newpath),
            || unsafe { redhook::real!(rename)(oldpath, newpath) }
        )
    }
}

redhook::hook! {
    unsafe fn stat(path: *const c_char, buf: *mut libc::stat) -> c_int => protect_stat {
        with_hook_protection(
            || hook_protection::handle_stat(path),
            || unsafe { redhook::real!(stat)(path, buf) }
        )
    }
}

redhook::hook! {
    unsafe fn access(path: *const c_char, mode: c_int) -> c_int => protect_access {
        with_hook_protection(
            || hook_protection::handle_access(path, mode),
            || unsafe { redhook::real!(access)(path, mode) }
        )
    }
}

redhook::hook! {
    unsafe fn realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char => protect_realpath {
        with_hook_protection(
            || hook_protection::handle_realpath(path, resolved_path),
            || unsafe { redhook::real!(realpath)(path, resolved_path) }
        )
    }
}

redhook::hook! {
    unsafe fn read(fd: c_int, buf: *mut libc::c_void, count: usize) -> isize => protect_read {
        with_hook_protection(
            || hook_protection::handle_read(fd, buf, count),
            || unsafe { redhook::real!(read)(fd, buf, count) }
        )
    }
}
