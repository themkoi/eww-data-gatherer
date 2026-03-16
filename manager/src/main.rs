use std::{env, error::Error, thread};

use log::debug;
mod actions;
pub mod config;
mod listeners;
mod manager;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "off"),
    );
    let _cfg = config::get_config();
    let args = env::args().collect::<Vec<String>>();

    let arg = &args[1];
    if arg == "-h" || arg == "--help" {
        return Ok(());
    } else if arg == "gatherer" {
        let listener_args = &args[2..];
        let all = listener_args.len() == 1 && listener_args[0] == "all";

        let listeners_to_run: Vec<String> = if all {
            vec![
                "brightness",
                "network",
                "player",
                "volume",
                "auto_idle",
                "bluetooth",
                "power_profile",
                "hid_bat",
                "output_audio",
                "playback_audio",
                "input_audio",
                "amd_gpu",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect()
        } else {
            listener_args.iter().map(|s| s.to_string()).collect()
        };

        let mut handles = Vec::new();

        for listener in listeners_to_run {
            debug!("starting gatherer {}", listener.as_str());

            let handle = match listener.as_str() {
                "brightness" => Some(thread::spawn(|| listeners::brightness::run())),
                "network" => Some(thread::spawn(|| listeners::network::run())),
                "player" => Some(thread::spawn(|| listeners::player::run())),
                "volume" => Some(thread::spawn(|| listeners::volume::run())),
                "auto_idle" => Some(thread::spawn(|| listeners::auto_idle::run())),
                "bluetooth" => Some(thread::spawn(|| listeners::bluetooth::run())),
                "power_profile" => Some(thread::spawn(|| listeners::power_profile::run())),
                "output_audio" => Some(thread::spawn(|| listeners::output_audio::run())),
                "hid_bat" => Some(thread::spawn(|| listeners::hid_bat::run())),
                "playback_audio" => Some(thread::spawn(|| listeners::playback_audio::run())),
                "input_audio" => Some(thread::spawn(|| listeners::input_audio::run())),
                "amd_gpu" => Some(thread::spawn(|| listeners::amd_gpu::run())),
                _ => {
                    debug!("unknown gatherer: {}", listener.as_str());
                    None
                }
            };

            if let Some(h) = handle {
                handles.push(h);
            }
        }

        for h in handles {
            let _ = h.join();
        }
    } else if arg == "action" {
        let arg = &args[2];
        println!("running action {}", arg);
        if arg == "toggleIdle" {
            actions::toggle_idle::action();
        } else if arg == "toggleWinow" {
            let winows = &args[3..];
            actions::toggle_window::action(winows);
        }
    } else if arg == "manager" {
        manager::run()?;
    } else {
        println!("Message sent");
    }

    Ok(())
}
