use std::process::{exit, Command};
use std::thread;
use std::time::Duration;

use crate::config::get_config;

fn run(cmd: &str, args: &[&str]) -> String {
    println!("[run] {} {:?}", cmd, args);

    let out = Command::new(cmd).args(args).output().unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

    if !stdout.is_empty() {
        println!("[run stdout] {}", stdout);
    }
    if !stderr.is_empty() {
        println!("[run stderr] {}", stderr);
    }

    stdout
}

fn run_bg(cmd: &str, args: &[&str]) {
    println!("[run] {} {:?}", cmd, args);
    Command::new(cmd).args(args).spawn().unwrap();
}

fn get_active_windows() -> Vec<String> {
    let out = run("eww", &["active-windows"]);

    out.lines()
        .filter_map(|l| l.split(':').next())
        .map(|s| s.trim().to_string())
        .collect()
}

fn window_is_open(active: &[String], name: &str) -> bool {
    active.iter().any(|w| w == name)
}

fn open_module(module: &str, active: &[String]) {
    let cfg = get_config();
    let modules = cfg.modules.clone();
    let anim = cfg.anim_duration;
    if !window_is_open(active, "closer") {
        run_bg("eww", &["open", "closer"]);
    }
    thread::sleep(Duration::from_millis(10));

    let state_var = format!("open_{}", module);

    // open window if needed
    if !window_is_open(active, module) {
        run_bg("eww", &["open", module]);
    }
    thread::sleep(Duration::from_millis(anim.into()));

    // update state
    run("eww", &["update", &format!("{}=true", state_var)]);

    // close other module states
    let mut args = vec!["update".to_string()];

    args.extend(
        modules
            .iter()
            .filter(|m| *m != module)
            .map(|m| format!("open_{}=false", m)),
    );

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    run("eww", &args_ref);

    // close other open windows
    let mut close_args = vec!["close"];

    for m in modules.iter().filter(|m| m.as_str() != module) {
        if window_is_open(active, m) {
            close_args.push(m);
        }
    }

    if close_args.len() > 1 {
        thread::sleep(Duration::from_millis(anim.into()));
        run("eww", &close_args);
    }
}

fn close_module(module: &str, active: &[String]) {
    let cfg = get_config();
    let anim = cfg.anim_duration;

    let state_var = format!("open_{}", module);

    run("eww", &["update", &format!("{}=false", state_var)]);

    if window_is_open(active, "closer") {
        run("eww", &["close", "closer"]);
    }

    if window_is_open(active, module) {
        thread::sleep(Duration::from_millis(anim.into()));
        run("eww", &["close", module]);
    }
}

pub fn action(args: &[String]) {
    println!("[action] args: {:?}", args);

    if std::fs::exists("/tmp/ewwManager-window.lock").unwrap_or(false) {
        println!("lock exists exiting");
        exit(0);
    }

    println!("creating lock");
    let _ = std::fs::File::create("/tmp/ewwManager-window.lock");

    let cfg = get_config();
    let modules = cfg.modules.clone();
    let anim = cfg.anim_duration;

    let active = get_active_windows();

    let module = args.get(0).cloned();

    if module.is_none() {
        println!("[action] closing all");

        if window_is_open(&active, "closer") {
            run("eww", &["close", "closer"]);
        }
        let mut args = vec!["update".to_string()];

        args.extend(modules.iter().map(|m| format!("open_{}=false", m)));

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        run("eww", &args_ref);

        thread::sleep(Duration::from_millis(anim.into()));

        let mut close_args = vec!["close"];

        for m in &modules {
            if window_is_open(&active, m) {
                close_args.push(m);
            }
        }

        if close_args.len() > 1 {
            run("eww", &close_args);
        }

        let _ = std::fs::remove_file("/tmp/ewwManager-window.lock");
        return;
    }

    let module = module.unwrap();
    println!("[action] module: {}", module);

    // unmanaged window → toggle
    if !modules.iter().any(|m| m == &module) {
        println!("[action] unmanaged window → toggling");

        if window_is_open(&active, &module) {
            run("eww", &["close", &module]);
        } else {
            run_bg("eww", &["open", &module]);
        }

        let _ = std::fs::remove_file("/tmp/ewwManager-window.lock");
        return;
    }

    if window_is_open(&active, &module) {
        println!("[action] closing {}", module);
        close_module(&module, &active);
    } else {
        println!("[action] opening {}", module);
        open_module(&module, &active);
    }

    println!("removing lock");
    let _ = std::fs::remove_file("/tmp/ewwManager-window.lock");
}
