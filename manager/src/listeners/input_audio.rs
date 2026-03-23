use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
struct AudioSource {
    index: u32,
    name: String,
    description: String,
    icon: String,
    muted: bool,
    volume: i32,
    is_default: bool,
}

fn get_sources() -> Vec<AudioSource> {
    let default_source = Command::new("pactl")
        .args(["info"])
        .output()
        .ok()
        .and_then(|out| {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .find(|l| l.starts_with("Default Source:"))
                .map(|l| l["Default Source:".len()..].trim().to_string())
        });

    let output = Command::new("pactl")
        .args(["list", "sources"])
        .output()
        .expect("Failed to run pactl");

    let s = String::from_utf8_lossy(&output.stdout);

    let mut sources = vec![];
    let mut current = AudioSource {
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

        if line.starts_with("Source #") {
            if !current.name.is_empty() {
                sources.push(current.clone());
            }

            current = AudioSource {
                index: line["Source #".len()..].parse().unwrap_or(0),
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
        sources.push(current);
    }

    if let Some(default) = default_source {
        for source in &mut sources {
            source.is_default = source.name == default;
        }
    }

    sources
}

#[derive(serde::Serialize)]
struct SourcesWrapper {
    sources: Vec<AudioSource>,
}

fn print_sources_json(sources: &[AudioSource], limit: usize) {
    let shown: Vec<AudioSource> = sources.iter().take(limit).cloned().collect();
    let wrapper = SourcesWrapper { sources: shown };

    println!("{}", &serde_json::to_string(&wrapper).unwrap());
    std::io::stdout().flush().unwrap();
}

fn only_volume_changed(old: &[AudioSource], new: &[AudioSource]) -> bool {
    if old.len() != new.len() {
        return false;
    }

    for new_source in new {
        let Some(old_source) = old.iter().find(|s| s.index == new_source.index) else {
            return false;
        };

        if old_source.name != new_source.name
            || old_source.description != new_source.description
            || old_source.muted != new_source.muted
            || old_source.is_default != new_source.is_default
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

    let mut last_state = get_sources();
    print_sources_json(&last_state, 10);

    let mut last_change = Instant::now();
    let mut pending = false;

    loop {
        if let Some(Ok(line)) = lines.next() {
            if line.contains("source") || line.contains("server") {
                let new_state = get_sources();

                if new_state != last_state {
                    let volume_only = only_volume_changed(&last_state, &new_state);

                    last_state = new_state;

                    if volume_only {
                        last_change = Instant::now();
                        pending = true;
                    } else {
                        print_sources_json(&last_state, 10);
                        pending = false;
                    }
                }
            }
        }

        if pending && last_change.elapsed() >= debounce {
            print_sources_json(&last_state, 10);
            pending = false;
        }

        sleep(Duration::from_millis(40));
    }
}