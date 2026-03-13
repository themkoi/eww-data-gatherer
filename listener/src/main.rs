use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;

fn listen_to_socket(name: &str) -> std::io::Result<()> {
    let path = "/tmp/EwwManager_".to_string() + name + ".sock";

    if fs::metadata(&path).is_ok() {
        let _ = fs::remove_file(&path);
    }

    let listener = UnixListener::bind(&path)?;

    for stream in listener.incoming() {
        let stream = stream?;
        let reader = BufReader::with_capacity(256, stream);

        for line in reader.lines() {
            println!(" {}", line?);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args().nth(1).unwrap();
    listen_to_socket(&arg)?;
    Ok(())
}