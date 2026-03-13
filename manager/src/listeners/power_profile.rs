use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::Serialize;

use crate::listeners::send_to_socket;

#[derive(Serialize)]
struct PowerInfo {
    profile: String,
}

fn get_profile() {
    let output = Command::new("powerprofilesctl")
        .arg("get")
        .output()
        .unwrap();

    let profile = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    let info = PowerInfo { profile };
    send_to_socket("power_profile", &serde_json::to_string(&info).unwrap()).unwrap();
}

pub fn run() {
    get_profile();

    let mut dbus = Command::new("dbus-monitor")
        .args(&[
            "--system",
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',sender='net.hadess.PowerProfiles'"
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = dbus.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("ActiveProfile") {
                get_profile();
            }
        }
    }
}
