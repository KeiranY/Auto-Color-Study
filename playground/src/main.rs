
use std::ffi::{c_void, c_char, c_int, c_uint};

mod utils;

fn hook1(arg: u8, chain: &mut putchar::Chain) -> (u8,) {
    println!("hook1 called with arg: {}", arg);
    let arg = arg + 1;
    println!("hook1 passing to next hook: {}", arg);
    let result = chain.call(arg);
    println!("hook1 received from next hook: {}", result.0);
    return result;
}

fn hook2(arg: u8, chain: &mut putchar::Chain) -> (u8,) {
    println!("hook2 called with arg: {}", arg);
    let arg = arg * 2;
    println!("hook2 passing to next hook: {}", arg);
    let result = chain.call(arg);
    println!("hook2 received from next hook: {}", result.0);
    let result = (result.0 + 1,);
    println!("hook2 modifying result: {}", result.0);
    return result;
}

// Generate the hook function using the macro
create_hook!(putchar(arg: u8) -> (u8,));

fn main() {
    putchar::add_hook(hook1);
    putchar::add_hook(hook2);
    // Call the original function
    println!("Hooked Result: {:?}", unsafe { putchar(42) });
    unsafe { open(b"test.txt\0".as_ptr() as *const c_char, 0) };
}


create_hooks!(
    open(cpath: *const c_char, oflag: c_int) -> c_int,
    open64(cpath: *const c_char, oflag: c_int) -> c_int,
    openat(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int,
    openat64(dirfd: c_int, cpath: *const c_char, oflag: c_int) -> c_int,
    fopen(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE,
    fopen64(cpath: *const c_char, mode: *const c_char) -> *mut libc::FILE,
    rename(oldpath: *const c_char, newpath: *const c_char) -> c_int,
    access(path: *const c_char, mode: c_int) -> c_int,
    realpath(path: *const c_char, resolved_path: *mut c_char) -> *mut c_char,
    read(fd: c_int, buf: *mut libc::c_void, count: usize) -> isize,
    pread(fd: c_int, buf: *mut libc::c_void, count: usize, offset: libc::off_t) -> isize,
    chmod(path: *const c_char, mode: libc::mode_t) -> c_int,
    fchmodat(dirfd: c_int, path: *const c_char, mode: libc::mode_t, flags: c_int) -> c_int,
    fchmod(fd: c_int, mode: libc::mode_t) -> c_int,
    unlink(path: *const c_char) -> c_int,
    unlinkat(dirfd: c_int, path: *const c_char, flags: c_int) -> c_int,
    renameat(olddirfd: c_int, oldpath: *const c_char, newdirfd: c_int, newpath: *const c_char) -> c_int,
    stat(path: *const c_char, buf: *mut libc::stat) -> c_int,
    _xstat(ver: c_int, path: *const c_char, buf: *mut libc::stat) -> c_int,
    statx(dirfd: c_int, path: *const c_char, flags: c_int, mask: c_uint, buf: *mut libc::statx) -> c_int,
    lstat(path: *const c_char, buf: *mut libc::stat) -> c_int,
    _lxstat(ver: c_int, path: *const c_char, buf: *mut libc::stat) -> c_int,
    fstat(fd: c_int, buf: *mut libc::stat) -> c_int,
    fstatat(dirfd: c_int, path: *const c_char, buf: *mut libc::stat, flags: c_int) -> c_int,
    _fxstat(ver: c_int, fd: c_int, buf: *mut libc::stat) -> c_int,
    faccessat(dirfd: c_int, path: *const c_char, mode: c_int, flags: c_int) -> c_int,
    readdir(dirp: *mut libc::DIR) -> *mut libc::dirent,
    readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64,
    scandir(dir: *const c_char, namelist: *mut *mut *mut libc::dirent, filter: Option<unsafe extern "C" fn(*const libc::dirent) -> c_int>, compar: Option<unsafe extern "C" fn(*const libc::dirent, *const libc::dirent) -> c_int>) -> c_int,
    scandir64(dir: *const c_char, namelist: *mut *mut *mut libc::dirent64, filter: Option<unsafe extern "C" fn(*const libc::dirent64) -> c_int>, compar: Option<unsafe extern "C" fn(*const libc::dirent64, *const libc::dirent64) -> c_int>) -> c_int
);
