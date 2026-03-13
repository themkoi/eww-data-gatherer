use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::listeners::send_to_socket;

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
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
            if let Some(percent) = line.split('/').nth(1) {
                current.volume = percent.trim().trim_end_matches('%').parse().unwrap_or(0);
            }
        }
    }

    if !current.name.is_empty() {
        sinks.push(current);
    }

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
}

fn print_sinks_json(sinks: &[AudioSink], limit: usize) {
    let shown: Vec<AudioSink> = sinks.iter().take(limit).cloned().collect();
    let wrapper = SinksWrapper { sinks: shown };

    send_to_socket("output_audio", &serde_json::to_string(&wrapper).unwrap()).unwrap();
    std::io::stdout().flush().unwrap();
}

fn only_volume_changed(old: &[AudioSink], new: &[AudioSink]) -> bool {
    if old.len() != new.len() {
        return false;
    }

    for new_sink in new {
        let Some(old_sink) = old.iter().find(|s| s.index == new_sink.index) else {
            return false;
        };

        if old_sink.name != new_sink.name
            || old_sink.description != new_sink.description
            || old_sink.muted != new_sink.muted
            || old_sink.is_default != new_sink.is_default
        {
            return false;
        }
    }

    true
}

pub fn run() {
    let mut pactl = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run pactl subscribe");

    let stdout = pactl.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let debounce = Duration::from_millis(400);

    let mut last_state = get_sinks();
    print_sinks_json(&last_state, 10);

    let mut last_change = Instant::now();
    let mut pending = false;

    loop {
        if let Some(Ok(line)) = lines.next() {
            if line.contains("sink") || line.contains("server") {
                let new_state = get_sinks();

                if new_state != last_state {
                    let volume_only = only_volume_changed(&last_state, &new_state);

                    last_state = new_state;

                    if volume_only {
                        last_change = Instant::now();
                        pending = true;
                    } else {
                        print_sinks_json(&last_state, 10);
                        pending = false;
                    }
                }
            }
        }

        if pending && last_change.elapsed() >= debounce {
            print_sinks_json(&last_state, 10);
            pending = false;
        }

        sleep(Duration::from_millis(40));
    }
}