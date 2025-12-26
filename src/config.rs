use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, sync::OnceLock};

static CONFIG_STATIC: OnceLock<DaemonConfig> = OnceLock::new();

pub fn get_config() -> &'static DaemonConfig {
    CONFIG_STATIC.get_or_init(|| load_or_create_config().unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonConfig {
    pub brightness_path: String,
    pub idle_manager: String,
    pub ipc_socket: String,
}

fn default_config() -> DaemonConfig {
    DaemonConfig {
        brightness_path: "/sys/class/backlight/amdgpu_bl1".to_string(),
        idle_manager: "swayidle".to_string(),
        ipc_socket: "/tmp/eww-res-daemon.sock".to_string(),
    }
}

fn get_config_file() -> PathBuf {
    let mut path = config_dir().unwrap();
    path.push("eww-res-daemon");
    fs::create_dir_all(&path).unwrap();
    path.push("config.toml");
    path
}

fn write_config<P: AsRef<Path>>(path: P, config: &DaemonConfig) -> std::io::Result<()> {
    let toml_string = toml::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(path, toml_string)
}

pub fn load_or_create_config() -> Result<DaemonConfig, Box<dyn std::error::Error>> {
    let path = get_config_file();
    if !path.exists() {
        let default = default_config();
        write_config(&path, &default)?;
        return Ok(default);
    }

    let loaded = ConfigLoader::builder()
        .add_source(File::with_name(path.to_str().unwrap()))
        .build()?
        .try_deserialize::<DaemonConfig>()?;

    Ok(loaded)
}
