use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize)]
struct SinkInput {
    index: u32,
    name: String,
    media_name: String,
    volume: u32,
    muted: bool,
    sink_id: u32,
    sink_name: String,
}

fn truncate(s: &str, max: usize) -> String {
    let len = s.chars().count();
    if len <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{}..", truncated)
}

fn parse_sinks() -> HashMap<u32, String> {
    let output = Command::new("pactl")
        .args(["list", "sinks"])
        .output()
        .expect("Failed to run pactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut sinks = HashMap::new();
    let mut index = 0;
    let mut description = String::new();

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("Sink #") {
            if index != 0 {
                sinks.insert(index, description.clone());
            }

            index = line["Sink #".len()..].parse().unwrap_or(0);
            description.clear();
        } else if line.starts_with("Description: ") {
            description = line["Description: ".len()..].to_string();
        }
    }

    if index != 0 {
        sinks.insert(index, description);
    }

    sinks
}

fn parse_sink_inputs() -> HashMap<u32, SinkInput> {
    let sink_map = parse_sinks();

    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
        .expect("Failed to run pactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut sinks = HashMap::new();
    let mut index = 0;
    let mut name = String::new();
    let mut media_name = String::new();
    let mut volume = 0;
    let mut muted = false;
    let mut sink_id = 0;

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("Sink Input #") {
            if index != 0 {
                sinks.insert(
                    index,
                    SinkInput {
                        index,
                        name: name.clone(),
                        media_name: media_name.clone(),
                        volume,
                        muted,
                        sink_id,
                        sink_name: sink_map
                            .get(&sink_id)
                            .cloned()
                            .unwrap_or_else(|| "unknown".into()),
                    },
                );
            }

            index = line["Sink Input #".len()..].parse().unwrap_or(0);
            name.clear();
            media_name.clear();
            volume = 0;
            muted = false;
            sink_id = 0;
        } else if line.starts_with("Mute: ") {
            muted = &line["Mute: ".len()..] == "yes";
        } else if line.starts_with("Volume: ") {
            if let Some(v) = line.split_whitespace().nth(4) {
                volume = v.trim_end_matches('%').parse().unwrap_or(0);
            }
        } else if line.starts_with("Sink: ") {
            sink_id = line["Sink: ".len()..].parse().unwrap_or(0);
        } else if line.starts_with("application.process.binary = ") {
            name = line["application.process.binary = ".len()..]
                .trim()
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("media.name = ") {
            media_name = line["media.name = ".len()..]
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }

    if index != 0 {
        sinks.insert(
            index,
            SinkInput {
                index,
                name,
                media_name,
                volume,
                muted,
                sink_id,
                sink_name: sink_map
                    .get(&sink_id)
                    .cloned()
                    .unwrap_or_else(|| "unknown".into()),
            },
        );
    }

    sinks
}

fn print_state(map: &HashMap<u32, SinkInput>) {
    let mut sinks: Vec<_> = map.values().cloned().collect();
    sinks.sort_by_key(|s| s.index);

    let all_sinks = parse_sinks();
    let mut enriched = Vec::new();

    for s in sinks {
        let mut ordered: Vec<(u32, String)> = all_sinks
            .iter()
            .map(|(id, name)| (*id, name.clone()))
            .collect();

        ordered.sort_by_key(|(id, _)| if *id == s.sink_id { 0 } else { 1 });

        let sink_names: Vec<String> = ordered
            .iter()
            .map(|(_, name)| truncate(name, 34))
            .collect();

        let commands: Vec<String> = ordered
            .iter()
            .map(|(id, _)| format!("pactl move-sink-input {} {}", s.index, id))
            .collect();

        enriched.push(json!({
            "index": s.index,
            "name": s.name,
            "media_name": s.media_name,
            "volume": s.volume,
            "muted": s.muted,
            "sink_id": s.sink_id,
            "sink_name": s.sink_name,
            "available_sinks": sink_names,
            "switch_commands": commands
        }));
    }

    let out = json!({
        "playbacks": enriched,
    });

    println!("{}", &serde_json::to_string(&out).unwrap());
}

fn only_volume_changed(
    old: &HashMap<u32, SinkInput>,
    new: &HashMap<u32, SinkInput>,
) -> bool {
    if old.len() != new.len() {
        return false;
    }

    for (idx, new_s) in new {
        let Some(old_s) = old.get(idx) else {
            return false;
        };

        if old_s.name != new_s.name
            || old_s.media_name != new_s.media_name
            || old_s.muted != new_s.muted
            || old_s.sink_id != new_s.sink_id
            || old_s.sink_name != new_s.sink_name
        {
            return false;
        }
    }

    true
}

pub fn run() {
    let mut child = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .expect("pactl subscribe failed");

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let debounce = Duration::from_millis(400);

    let mut last_state = parse_sink_inputs();
    print_state(&last_state);

    let mut last_change = Instant::now();
    let mut pending = false;

    loop {
        if let Some(Ok(line)) = lines.next() {
            if line.contains("sink-input") || line.contains("sink") {
                let new_state = parse_sink_inputs();

                if new_state != last_state {
                    let volume_only = only_volume_changed(&last_state, &new_state);

                    last_state = new_state;

                    if volume_only {
                        last_change = Instant::now();
                        pending = true;
                    } else {
                        print_state(&last_state);
                        pending = false;
                    }
                }
            }
        }

        if pending && last_change.elapsed() >= debounce {
            print_state(&last_state);
            pending = false;
        }

        sleep(Duration::from_millis(40));
    }
}