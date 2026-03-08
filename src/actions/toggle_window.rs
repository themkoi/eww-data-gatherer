use std::process::Command;
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
    println!("[run_bg] {} {:?}", cmd, args);

    match Command::new(cmd).args(args).spawn() {
        Ok(_) => {}
        Err(e) => println!("[run_bg error] {}", e),
    }
}

fn open_module(module: &str) {
    let cfg = get_config();
    let modules = cfg.modules.clone();
    let anim = cfg.anim_duration;

    let state_var = format!("open_{}", module);

    // open closer first
    run_bg("eww", &["open", "closer"]);

    // check if window exists
    let windows = run("eww", &["list-windows"]);
    if !windows.contains(&format!("*{}", module)) {
        run_bg("eww", &["open", module]);
        thread::sleep(Duration::from_millis(50));
    }

    run_bg("eww", &["update", &format!("{}=true", state_var)]);

    // close other modules
    for m in &modules {
        if m != module {
            run_bg("eww", &["update", &format!("open_{}=false", m)]);
        }
    }

    // wait for animation duration and close other windows
    thread::sleep(Duration::from_secs_f32(anim));
    for m in &modules {
        if m != module {
            run_bg("eww", &["close", m]);
        }
    }
}

fn close_module(module: &str) {
    let cfg = get_config();
    let anim = cfg.anim_duration;
    let state_var = format!("open_{}", module);

    run_bg("eww", &["update", &format!("{}=false", state_var)]);
    run_bg("eww", &["close", "closer"]);
    thread::sleep(Duration::from_secs_f32(anim));
    run_bg("eww", &["close", module]);
}

pub fn action(args: &[String]) {
    println!("[action] args: {:?}", args);

    let cfg = get_config();
    let modules = cfg.modules.clone();
    let anim = cfg.anim_duration;

    let module = args.get(0).cloned(); // Option<String>

    if module.is_none() {
        println!("[action] closing all");

        run_bg("eww", &["close", "closer"]);

        for m in &modules {
            run_bg("eww", &["update", &format!("open_{}=false", m)]);
        }

        thread::spawn(move || {
            thread::sleep(Duration::from_secs_f32(anim));
            for m in &modules {
                run_bg("eww", &["close", m]);
            }
        });

        return;
    }

    let module = module.unwrap();
    println!("[action] module: {}", module);

    // If it's not a managed module, just toggle it
    if !modules.iter().any(|m| m == &module) {
        println!("[action] unmanaged window → toggling");

        let windows = run("eww", &["list-windows"]);

        if windows.contains(&format!("*{}", module)) {
            run_bg("eww", &["close", &module]);
        } else {
            run_bg("eww", &["open", &module]);
        }

        return;
    }

    let state_var = format!("open_{}", module);
    let current = run("eww", &["get", &state_var]);

    println!("[action] state {} = {}", state_var, current);

    let is_open = current.trim().eq_ignore_ascii_case("true");

    if is_open {
        println!("[action] closing {}", module);
        let handle = thread::spawn(move || close_module(&module));
        handle.join().unwrap();
    } else {
        println!("[action] opening {}", module);
        let handle = thread::spawn(move || open_module(&module));
        handle.join().unwrap();
    }
}
