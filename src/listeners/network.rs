use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct WifiStatus {
    essid: String,
    signal: String,
}

fn get_wifi_status() -> WifiStatus {
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
    let mut ip_monitor = Command::new("ip")
        .args(&["monitor", "link"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = ip_monitor.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let status = get_wifi_status();
    println!("{}", serde_json::to_string(&status).unwrap());
    std::io::stdout().flush().unwrap();

    for _line in reader.lines() {
        let status = get_wifi_status();
        println!("{}", serde_json::to_string(&status).unwrap());
        std::io::stdout().flush().unwrap();
    }
}
