use std::{io::{Read, Write}, os::unix::net::UnixStream};

pub mod toggle_idle;

use crate::config;

fn send_to_ipc(msg: String) -> std::io::Result<String> {
    let cfg = config::get_config();
    let mut stream = UnixStream::connect(cfg.ipc_socket.clone())?;
    stream.write_all(msg.as_bytes())?;

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

pub fn send_to_ipc_async(msg: String) {
    std::thread::spawn(move || {
        let _ = send_to_ipc(msg);
    });
}
