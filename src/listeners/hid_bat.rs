use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[derive(Serialize, Clone, Default)]
struct HidBattery {
    device: String,
    model: String,
    percentage: u8,
    state: String,
}

#[derive(Serialize)]
struct Devices {
    devices: Vec<HidBattery>,
    remaining: usize,
}

// Parse a single UPower device (HID battery)
fn parse_device_info(path: &str) -> Option<HidBattery> {
    let output = Command::new("upower").args(&["-i", path]).output().ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut device = HidBattery::default();
    device.device = path.to_string();

    for line in text.lines() {
        let l = line.trim();

        if l.starts_with("model:") {
            device.model = l.replace("model:", "").trim().to_string();
        }

        if l.starts_with("percentage:") {
            if let Ok(num) = l
                .replace("percentage:", "")
                .replace("%", "")
                .trim()
                .parse::<u8>()
            {
                device.percentage = num;
            }
        }

        if l.starts_with("state:") {
            device.state = l.replace("state:", "").trim().to_string();
        }
    }

    Some(device)
}

fn get_all_devices() -> Vec<HidBattery> {
    let output = Command::new("upower").arg("-e").output().unwrap();

    let list = String::from_utf8_lossy(&output.stdout);

    list.lines()
        .filter(|p| p.contains("battery") && !p.contains("battery_BAT")) // skip internal batteries
        .filter_map(parse_device_info)
        .collect()
}

// Print devices as JSON, limited to `limit`, with `remaining` count
fn print_all(limit: usize) {
    // Uncomment below to use dummy devices for testing
    // let all = vec![
    //     HidBattery { device: "dev1".to_string(), model: "G903".to_string(), percentage: 87, state: "discharging".to_string() },
    //     HidBattery { device: "dev2".to_string(), model: "MX Keys".to_string(), percentage: 54, state: "charging".to_string() },
    //     HidBattery { device: "dev3".to_string(), model: "M720".to_string(), percentage: 32, state: "discharging".to_string() },
    // ];

    let all = get_all_devices();

    let shown: Vec<HidBattery> = all.iter().take(limit).cloned().collect();
    let remaining = all.len().saturating_sub(limit);

    let wrapper = Devices {
        devices: shown,
        remaining,
    };
    println!("{}", serde_json::to_string(&wrapper).unwrap());
    std::io::stdout().flush().unwrap();
}

pub fn run() {
    let limit = 2;

    // Print all devices initially
    print_all(limit);

    // Monitor UPower for changes
    let mut upower = Command::new("upower")
        .args(&["--monitor"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn upower");

    let stdout = upower.stdout.take().expect("stdout not captured");
    let reader = BufReader::new(stdout);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if line.contains("changed") {
                print_all(limit);
            }
        }
    }
}
