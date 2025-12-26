use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::Serialize;

#[derive(Serialize)]
struct BtInfo {
    on: bool,
    name: String,
    signal: String,
}

fn get_bt() {
    // Check if Bluetooth is powered
    let output = Command::new("bluetoothctl")
        .arg("show")
        .output()
        .unwrap();
    let binding = String::from_utf8_lossy(&output.stdout);
    let powered = binding
        .lines()
        .find(|line| line.contains("Powered:"))
        .map(|line| line.split_whitespace().last().unwrap_or(""))
        .unwrap_or("");

    if powered != "yes" {
        let info = BtInfo { on: false, name: "".into(), signal: "".into() };
        println!("{}", serde_json::to_string(&info).unwrap());
        return;
    }

    // Find first connected device
    let output = Command::new("bluetoothctl")
        .arg("devices")
        .output()
        .unwrap();
    let mut mac = String::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(addr) = line.split_whitespace().nth(1) {
            let info_out = Command::new("bluetoothctl")
                .arg("info")
                .arg(addr)
                .output()
                .unwrap();
            let info_text = String::from_utf8_lossy(&info_out.stdout);
            if info_text.contains("Connected: yes") {
                mac = addr.to_string();
                break;
            }
        }
    }

    if mac.is_empty() {
        let info = BtInfo { on: true, name: "".into(), signal: "".into() };
        println!("{}", serde_json::to_string(&info).unwrap());
        return;
    }

    // Get device name and RSSI
    let info_out = Command::new("bluetoothctl")
        .arg("info")
        .arg(&mac)
        .output()
        .unwrap();
    let info_text = String::from_utf8_lossy(&info_out.stdout);

    let name = info_text
        .lines()
        .find(|line| line.contains("Name:"))
        .map(|line| line.trim_start().trim_start_matches("Name:").trim().to_string())
        .unwrap_or_default();

    let rssi = info_text
        .lines()
        .find(|line| line.contains("RSSI"))
        .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
        .unwrap_or_default();

    let info = BtInfo { on: true, name, signal: rssi };
    println!("{}", serde_json::to_string(&info).unwrap());
}

pub fn run() {
    get_bt();

    let mut dbus = Command::new("dbus-monitor")
        .args(&[
            "--system",
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',sender='org.bluez'"
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = dbus.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("RSSI") || line.contains("Connected") || line.contains("Powered") {
                get_bt();
            }
        }
    }
}
