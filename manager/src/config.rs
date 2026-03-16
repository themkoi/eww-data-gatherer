use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, sync::OnceLock};

static CONFIG_STATIC: OnceLock<DaemonConfig> = OnceLock::new();

pub fn get_config() -> &'static DaemonConfig {
    CONFIG_STATIC.get_or_init(|| load_or_create_config().unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonConfig {
    pub brightness_path: String,
    pub idle_manager: String,
    pub idle_start_script: String,
    pub ipc_socket: String,
    pub com_on_output: String,

    pub modules: Vec<String>,
    pub anim_duration: u16,

    pub gpus: Vec<GpuDevice>,
}

fn default_config() -> DaemonConfig {
    DaemonConfig {
        brightness_path: "/sys/class/backlight/amdgpu_bl1".to_string(),
        idle_manager: "swayidle".to_string(),
        idle_start_script: "/Documents/scripts/niri/launch-idle-manage.sh".to_string(),
        ipc_socket: "/tmp/eww-res-daemon.sock".to_string(),
        com_on_output: "eww reload".to_string(),

        modules: vec![
            "control_center".to_string(),
            "date".to_string(),
            "power_menu".to_string(),
        ],
        anim_duration: 200,

        gpus: vec![
            GpuDevice {
                name: "amd_radeon_780m_graphics".to_string(),
                path: "/sys/class/drm/card1/device".to_string(),
            },
        ],
    }
}

fn get_config_file() -> PathBuf {
    let mut path = config_dir().unwrap();
    path.push("eww-manager");
    fs::create_dir_all(&path).unwrap();
    path.push("config.toml");
    path
}

fn write_config<P: AsRef<Path>>(path: P, config: &DaemonConfig) -> std::io::Result<()> {
    let toml_string = toml::to_string_pretty(config).unwrap();
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
        .add_source(File::from(path.clone()))
        .build()?
        .try_deserialize::<DaemonConfig>()?;

    Ok(loaded)
}