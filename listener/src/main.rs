use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

fn handle_client(stream: UnixStream) {
    let reader = BufReader::new(stream);

    for line in reader.lines().flatten() {
        if !line.trim().is_empty() {
            println!("{line}");
            io::stdout().flush().unwrap();
        }
    }
}

fn listen_to_socket(name: &str) -> io::Result<()> {
    let path = format!("/tmp/EwwManager_{}.sock", name);

    if fs::metadata(&path).is_ok() {
        let _ = fs::remove_file(&path);
    }

    let listener = UnixListener::bind(&path)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => eprintln!("connection error: {e}"),
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args().nth(1).expect("Socket name required");
    listen_to_socket(&arg)?;
    Ok(())
}