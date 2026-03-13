use crate::config;
use crate::listeners::send_to_socket;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;

pub fn run() {
    let cfg = config::get_config();

    let _ = std::fs::remove_file(&cfg.ipc_socket);

    let listener = match UnixListener::bind(&cfg.ipc_socket) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind socket: {}", e);
            return;
        }
    };

    println!("Listening on {}", cfg.ipc_socket);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                continue;
            }
        };

        let reader = BufReader::new(stream);

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to read line: {}", e);
                    continue;
                }
            };

            match line.trim() {
                "autoidle: true" => send_to_socket("auto_idle", "true").unwrap(),
                "autoidle: false" => send_to_socket("auto_idle", "false").unwrap(),
                _ => continue,
            }
        }
    }
}
