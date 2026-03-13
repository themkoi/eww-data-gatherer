use std::thread;
use std::time::Duration;
use std::{os::unix::net::UnixStream, path::PathBuf};
use std::io::Write;

pub mod volume;
pub mod brightness;
pub mod network;
pub mod player;
pub mod auto_idle;
pub mod bluetooth;
pub mod power_profile;
pub mod hid_bat;
pub mod output_audio;
pub mod input_audio;
pub mod playback_audio;
pub mod amd_gpu;

fn send_to_socket(name: &str, message: &str) -> std::io::Result<()> {
    let mut path = PathBuf::from("/tmp");
    path.push(format!("{}{}.sock", "EwwManager_", name));

    // Wait until a listener is ready
    let mut stream = loop {
        match UnixStream::connect(&path) {
            Ok(s) => break s, // connected successfully
            Err(e) if e.kind() == std::io::ErrorKind::NotFound
                     || e.kind() == std::io::ErrorKind::ConnectionRefused => {
                // socket not ready yet, wait a bit
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e), // other errors propagate
        }
    };

    writeln!(stream, "{message}")?;
    stream.flush()?;

    Ok(())
}