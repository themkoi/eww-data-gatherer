use notify_rust::Notification;
use std::{
    ffi::OsStr, process::{Command, Stdio}, thread
};
use sysinfo::{Signal, System};
use crate::{actions::{send_to_ipc_async}, config};

fn send_notification(title: &str, message: &str) {
    let title = title.to_string();
    let message = message.to_string();
    thread::spawn(move || {
        let _ = Notification::new()
            .summary(&title)
            .body(&message)
            .timeout(5000)
            .show();
    });
}

pub fn action() {
    let cfg = config::get_config();
    let idle_manager = cfg.idle_manager.clone();
    let mut system = System::new_all();
    system.refresh_all();

    let processes: Vec<_> = system
        .processes_by_name(OsStr::new(&idle_manager))
        .collect();

    if !processes.is_empty() {
        send_notification("swayidle Status", "swayidle is running. Terminating it...");
        for process in processes {
            let _ = process.kill_with(Signal::Term);
        }
        let _ = send_to_ipc_async("autoidle: false".to_string());
    } else {
        send_notification("swayidle Status", "swayidle is not running. Starting it...");
        let launch_script = format!(
            "{}{}",
            std::env::var("HOME").unwrap(),
            cfg.idle_start_script
        );

        thread::spawn(move || {
            let _ = Command::new("sh")
                .arg(&launch_script)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        });
        let _ = send_to_ipc_async("autoidle: true".to_string());
    }
}
