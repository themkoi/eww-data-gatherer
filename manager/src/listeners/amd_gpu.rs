use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::listeners::send_to_socket;

#[derive(Debug, Deserialize)]
struct AmdgpuTopRoot {
    devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
struct Device {
    #[serde(rename = "Info")]
    info: DeviceInfo,
    #[serde(rename = "Sensors")]
    sensors: Sensors,
    #[serde(rename = "VRAM")]
    vram: VramData,
    #[serde(rename = "gpu_activity")]
    gpu_activity: GpuActivity,
}

#[derive(Debug, Deserialize)]
struct DeviceInfo {
    #[serde(rename = "DeviceName")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct Sensors {
    #[serde(rename = "Edge Temperature")]
    edge_temp: ValueUnit<f64>,
}

#[derive(Debug, Deserialize)]
struct VramData {
    #[serde(rename = "Total VRAM")]
    total_vram: ValueUnit<u64>,
    #[serde(rename = "Total VRAM Usage")]
    total_vram_usage: ValueUnit<u64>,
}

#[derive(Debug, Deserialize)]
struct GpuActivity {
    #[serde(rename = "GFX")]
    gfx: ValueUnit<f64>,
}

#[derive(Debug, Deserialize)]
struct ValueUnit<T> {
    value: Option<T>,
}

#[derive(Serialize)]
struct GpuStats {
    name: String, // Keep the original name here
    load_pct: f64,
    vram_used_mib: u64,
    vram_total_mib: u64,
    temp_c: f64,
}

pub fn run() {
    let mut proc = Command::new("amdgpu_top")
        .arg("-J")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run amdgpu_top -J");

    let stdout = proc.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        if let Ok(json_line) = line {
            if let Ok(root) = serde_json::from_str::<AmdgpuTopRoot>(&json_line) {
                let mut output = BTreeMap::new();

                for device in root.devices {
                    // Create the clean key: lowercase and replace spaces with underscores
                    let clean_key = device.info.name
                        .to_lowercase()
                        .replace(' ', "_");

                    let stats = GpuStats {
                        name: device.info.name, // Original name
                        load_pct: device.gpu_activity.gfx.value.unwrap_or(0.0),
                        vram_used_mib: device.vram.total_vram_usage.value.unwrap_or(0),
                        vram_total_mib: device.vram.total_vram.value.unwrap_or(0),
                        temp_c: device.sensors.edge_temp.value.unwrap_or(0.0),
                    };
                    
                    output.insert(clean_key, stats);
                }

                if let Ok(final_json) = serde_json::to_string(&output) {
                    send_to_socket("amd_gpu", &final_json).unwrap();
                }
            }
        }
    }
}