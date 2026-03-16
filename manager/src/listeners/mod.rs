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

use std::{
    io::Write,
    os::unix::net::UnixStream,
    path::PathBuf,
    thread,
    time::Duration,
};

pub fn send_to_socket(name: &str, message: &str) -> std::io::Result<()> {
    let mut path = PathBuf::from("/tmp");
    path.push(format!("EwwManager_{}.sock", name));

    let mut stream = loop {
        match UnixStream::connect(&path) {
            Ok(s) => break s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound
                || e.kind() == std::io::ErrorKind::ConnectionRefused =>
            {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    };

    writeln!(stream, "{message}")?;
    stream.flush()?;
    Ok(())
}