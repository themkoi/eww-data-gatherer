use std::{env, error::Error};

use log::debug;
mod actions;
pub mod config;
mod listeners;
mod manager;

fn print_help() {
    println!("ewwManager {}", env!("CARGO_PKG_VERSION"));
    println!("Usage: ewwManager [OPTIONS] <COMMAND> <args>");
    println!();
    println!("Options:");
    println!("  -h, --help       Print this help message");
    println!("  -v, --version    Print version information");
    println!();
    println!("Commands:");
    println!("  listener <type>  Start a listener. Types: brightness, network, player, volume, auto_idle, bluetooth");
    println!("  action <name>    Perform an action. Supported: toggleIdle");
    println!("Other messages are simply sent with 'Message sent'");
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "off"),
    );
    let _cfg = config::get_config();
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let arg = &args[1];
    if arg == "-h" || arg == "--help" {
        print_help();
        return Ok(());
    } else if arg == "listener" {
        let arg = &args[2];
        debug!("starting listener {}", arg);
        if arg == "brightness" {
            listeners::brightness::run();
        } else if arg == "network" {
            listeners::network::run();
        } else if arg == "player" {
            listeners::player::run();
        } else if arg == "volume" {
            listeners::volume::run();
        } else if arg == "auto_idle" {
            listeners::auto_idle::run();
        } else if arg == "bluetooth" {
            listeners::bluetooth::run();
        } else if arg == "power_profile" {
            listeners::power_profile::run();
        } else if arg == "hid_bat" {
            listeners::hid_bat::run();
        } else if arg == "output_audio" {
            listeners::output_audio::run();
        } else if arg == "playback_audio" {
            listeners::playback_audio::run();
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
