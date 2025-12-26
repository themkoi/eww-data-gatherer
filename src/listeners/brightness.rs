use std::fs;
use std::process::{Command, Stdio};
use std::io::BufRead;
use std::io::BufReader;
use crate::config;

fn get_brightness() {
    let cfg = config::get_config();

    let cur: u32 = fs::read_to_string(format!("{}/brightness", cfg.brightness_path))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let max: u32 = fs::read_to_string(format!("{}/max_brightness", cfg.brightness_path))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);

    println!("{}", cur * 100 / max);
}

pub fn run() {
    get_brightness();

    let mut udevadm = Command::new("udevadm")
        .args(&["monitor", "--subsystem-match=backlight", "--property"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = udevadm.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut action_change = false;

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.trim() == "ACTION=change" {
                action_change = true;
            } else if line.is_empty() && action_change {
                get_brightness();
                action_change = false;
            }
        }
    }
}
