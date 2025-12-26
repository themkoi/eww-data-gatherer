use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct WifiStatus {
    essid: String,
    signal: String,
}

fn get_wifi_status() -> WifiStatus {
    // Get signal of currently active Wi-Fi
    let signal = Command::new("nmcli")
        .args(&["-f", "in-use,signal", "dev", "wifi"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| {
            s.lines()
                .find(|l| l.starts_with("*"))
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("0")
                .to_string()
        })
        .unwrap_or_else(|| "0".to_string());

    // Get currently active ESSID
    let essid = Command::new("nmcli")
        .args(&["-t", "-f", "NAME", "connection", "show", "--active"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().next().unwrap_or("").to_string())
        .unwrap_or_else(|| "".to_string());

    WifiStatus { essid, signal }
}

pub fn run() {
    // Print initial status immediately
    thread::sleep(Duration::from_millis(50));
    let status = get_wifi_status();
    {
        let mut out = std::io::stdout().lock();
        writeln!(out, "{}", serde_json::to_string(&status).unwrap()).unwrap();
        out.flush().unwrap();
    }

    // Small sleep to stabilize interfaces
    thread::sleep(Duration::from_millis(50));

    // Start ip monitor
    let mut ip_monitor = Command::new("ip")
        .args(&["monitor", "link"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = ip_monitor.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for _line in reader.lines() {
        // On any interface change, print updated status
        let status = get_wifi_status();
        let mut out = std::io::stdout().lock();
        writeln!(out, "{}", serde_json::to_string(&status).unwrap()).unwrap();
        out.flush().unwrap();
    }
}
