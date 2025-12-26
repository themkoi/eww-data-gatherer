use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use serde::Serialize;

#[derive(Serialize)]
struct TrackInfo {
    name: String,
    title: String,
    artist: String,
    art_url: String,
    status: String,
    length: String,
    length_str: String,
}

fn parse_track_info(raw: &str) -> TrackInfo {
    // Parse raw JSON using serde_json
    let v: serde_json::Value = serde_json::from_str(raw).unwrap_or_default();

    let length = v.get("length")
        .and_then(|l| l.as_i64())
        .map(|l| ((l + 500_000) / 1_000_000).to_string())
        .unwrap_or_default();

    let art_url = v.get("artUrl")
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .trim_start_matches("file://")
        .to_string();

    let length_str = Command::new("playerctl")
        .args(&["metadata", "-f", "{{duration(mpris:length)}}"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    TrackInfo {
        name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        title: v.get("title").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        artist: v.get("artist").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        art_url,
        status: v.get("status").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        length,
        length_str,
    }
}

pub fn run() {
    let mut playerctl = Command::new("playerctl")
        .args(&["metadata", "-F", "-f",
                r#"{"name":"{{playerName}}","title":"{{title}}","artist":"{{artist}}","artUrl":"{{mpris:artUrl}}","status":"{{status}}","length":"{{mpris:length}}"}"#])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = playerctl.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(raw) = line {
            let track = parse_track_info(&raw);
            println!("{}", serde_json::to_string(&track).unwrap());
            std::io::stdout().flush().unwrap();
        }
    }
}
