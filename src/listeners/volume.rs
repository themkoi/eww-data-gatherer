use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

fn get_volume() {
    let output = Command::new("pamixer")
        .arg("--get-volume-human")
        .output()
        .unwrap();

    let binding = String::from_utf8_lossy(&output.stdout);
    let vol = binding
        .trim()
        .trim_end_matches('%');

    println!("{vol}");
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
