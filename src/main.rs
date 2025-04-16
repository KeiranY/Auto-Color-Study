use std::env;

use libc::geteuid;
use log::{info, warn, error};

mod install;
mod daemon;

fn main() {
    colog::init();

    if unsafe { geteuid() != 0 } {
        warn!("😞 Not running as root, running in daemon mode.");
        daemon::main();
    }
    else if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_name) = exe_path.file_name() {
            if exe_name == "auto-color" {
                info!("✅ Started from known location, running in daemon mode.");
                daemon::main();
            } else {
                info!("🔧 Started as root, performing installation.");
                install::main();
            }
        } else {
            error!("⚠️ Failed to retrieve executable name.");
        }
    } else {
        error!("⚠️ Failed to retrieve current executable path.");
    }
}
