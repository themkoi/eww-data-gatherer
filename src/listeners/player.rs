use std::{thread, time::Duration, process::Command, io::{Write, stdout}};
use serde::Serialize;

#[derive(Serialize, Clone)]
struct TrackInfo {
    title: String,
    artist: String,
    length: i64,
    progress: i64,
    length_str: String,
    progress_str: String,
    art_url: String,
}

fn seconds_to_mmss(sec: i64) -> String {
    let m = sec / 60;
    let s = sec % 60;
    format!("{:01}:{:02}", m, s)
}

fn get_length() -> (i64, String) {
    let len_str = Command::new("playerctl")
        .args(&["metadata", "--format", "{{duration(mpris:length)}}"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    let len = {
        let parts: Vec<&str> = len_str.split(':').collect();
        if parts.len() == 2 {
            parts[0].parse::<i64>().unwrap_or(0) * 60 + parts[1].parse::<i64>().unwrap_or(0)
        } else { 0 }
    };

    (len, len_str)
}

fn get_progress() -> i64 {
    Command::new("playerctl")
        .args(&["position"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|f| f as i64)
        .unwrap_or(0)
}

fn get_art_url() -> String {
    Command::new("playerctl")
        .args(&["metadata", "--format", "{{mpris:artUrl}}"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn run() {
    let mut last_title = String::new();
    let mut last_artist = String::new();
    let mut last_length = 0;
    let mut last_length_str = String::new();
    let mut last_art_url = String::new();

    loop {
        let title = Command::new("playerctl")
            .args(&["metadata", "--format", "{{title}}"])
            .output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim().to_string();

        let artist = Command::new("playerctl")
            .args(&["metadata", "--format", "{{artist}}"])
            .output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim().to_string();

        let (length, length_str) = if title != last_title || artist != last_artist {
            get_length()
        } else {
            (last_length, last_length_str.clone())
        };

        let art_url = if title != last_title || artist != last_artist {
            get_art_url()
        } else {
            last_art_url.clone()
        };

        last_title = title.clone();
        last_artist = artist.clone();
        last_length = length;
        last_length_str = length_str.clone();
        last_art_url = art_url.clone();

        let progress = get_progress();

        let track = TrackInfo {
            title: title.clone(),
            artist: artist.clone(),
            length,
            progress,
            length_str: length_str.clone(),
            progress_str: seconds_to_mmss(progress),
            art_url,
        };

        println!("{}", serde_json::to_string(&track).unwrap());
        stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(500));
    }
}