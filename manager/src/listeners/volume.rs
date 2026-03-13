use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::listeners::send_to_socket;

fn get_volume() {
    let output = Command::new("pamixer")
        .arg("--get-volume-human")
        .output()
        .unwrap();

    let binding = String::from_utf8_lossy(&output.stdout);
    let vol = binding
        .trim()
        .trim_end_matches('%');

    send_to_socket("volume", &format!("{vol}")).unwrap();
}

pub fn run() {
    get_volume();

    let mut pactl = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = pactl.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("on sink") {
                get_volume();
            }
        }
    }
}
