use crate::config;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct GpuStats {
    name: String,
    load_pct: f64,
    vram_used_mib: u64,
    vram_total_mib: u64,
    temp_c: f64,
}

fn read_u64(path: &str) -> u64 {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn read_f64(path: &str) -> f64 {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0.0)
}

pub fn run() {
    let cfg = config::get_config();
    let gpus = &cfg.gpus;

    loop {
        let mut output = BTreeMap::new();

        for gpu in gpus {
            let load = read_f64(&format!("{}/gpu_busy_percent", gpu.path));

            let vram_used = read_u64(&format!("{}/mem_info_vram_used", gpu.path)) / 1024 / 1024;
            let vram_total = read_u64(&format!("{}/mem_info_vram_total", gpu.path)) / 1024 / 1024;

            let temp = read_u64(&format!("{}/hwmon/hwmon4/temp1_input", gpu.path)) as f64 / 1000.0;

            let stats = GpuStats {
                name: gpu.name.to_string(),
                load_pct: load,
                vram_used_mib: vram_used,
                vram_total_mib: vram_total,
                temp_c: temp,
            };

            output.insert(gpu.name.to_string(), stats);
        }

        if let Ok(final_json) = serde_json::to_string(&output) {
            println!("{}", &final_json);
        }

        thread::sleep(Duration::from_secs(1));
    }
}
