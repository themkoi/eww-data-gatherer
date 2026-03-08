use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug,Clone,serde::Serialize)]
struct AudioSink {
    index: u32,
    name: String,
    description: String,
    icon: String,
    muted: bool,
    volume: i32,
    is_default: bool,
}

fn get_sinks() -> Vec<AudioSink> {
    // Get default sink
    let default_sink = Command::new("pactl")
        .args(["info"])
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .find(|l| l.starts_with("Default Sink:"))
                .map(|l| l["Default Sink:".len()..].trim().to_string())
        });

    // Get sink list in a parseable format
    let output = Command::new("pactl")
        .args(["list", "sinks"])
        .output()
        .expect("Failed to run pactl");

    let s = String::from_utf8_lossy(&output.stdout);
    let mut sinks = vec![];
    let mut current = AudioSink {
        index: 0,
        name: "".into(),
        description: "".into(),
        icon: "audio-speakers".into(),
        muted: false,
        volume: 0,
        is_default: false,
    };

    for line in s.lines() {
        let line = line.trim();
        if line.starts_with("Sink #") {
            if !current.name.is_empty() {
                sinks.push(current.clone());
            }
            current = AudioSink {
                index: line["Sink #".len()..].parse().unwrap_or(0),
                name: "".into(),
                description: "".into(),
                icon: "audio-speakers".into(),
                muted: false,
                volume: 0,
                is_default: false,
            };
        } else if line.starts_with("Name:") {
            current.name = line["Name:".len()..].trim().to_string();
        } else if line.starts_with("Description:") {
            current.description = line["Description:".len()..].trim().to_string();
        } else if line.starts_with("Mute:") {
            current.muted = line["Mute:".len()..].trim() == "yes";
        } else if line.starts_with("Volume:") {
            // Example: Volume: front-left: 65536 / 100% / 0.00 dB, ...
            if let Some(percent) = line.split('/').nth(1) {
                current.volume = percent.trim().trim_end_matches('%').parse().unwrap_or(0);
            }
        }
    }
    if !current.name.is_empty() {
        sinks.push(current);
    }

    // mark default sink
    if let Some(default) = default_sink {
        for sink in &mut sinks {
            sink.is_default = sink.name == default;
        }
    }

    sinks
}

#[derive(serde::Serialize)]
struct SinksWrapper {
    sinks: Vec<AudioSink>,
    remaining: usize,
}

fn print_sinks_json(sinks: &[AudioSink], limit: usize) {
    let shown: Vec<AudioSink> = sinks.iter().take(limit).cloned().collect();
    let remaining = sinks.len().saturating_sub(limit);

    let wrapper = SinksWrapper { sinks: shown, remaining };

    println!("{}", serde_json::to_string(&wrapper).unwrap());
}


pub fn run() {
    print_sinks_json(&get_sinks(),10);

    let mut pactl = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run pactl subscribe");

    let stdout = pactl.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("on sink") || line.contains("server") {
                print_sinks_json(&get_sinks(),10);
            }
        }
    }
}