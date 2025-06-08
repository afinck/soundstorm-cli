use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use serde_json::Value;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::fs;
use toml::Value as TomlValue;

fn start_mpv(ipc_path: &str, url: &str) -> io::Result<Child> {
    Command::new("mpv")
        .arg(url)
        .arg(format!("--input-ipc-server={}", ipc_path))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

fn send_mpv_command(ipc_path: &str, command: &str) -> io::Result<()> {
    use std::os::unix::net::UnixStream;
    use std::io::Write;
    let mut stream = UnixStream::connect(ipc_path)?;
    let json = format!(r#"{{"command": [{}]}}"#, command);
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

fn get_mpv_property(ipc_path: &str, property: &str) -> io::Result<Option<String>> {
    use std::os::unix::net::UnixStream;
    use std::io::{Write, BufRead, BufReader};

    let mut stream = UnixStream::connect(ipc_path)?;
    let cmd = format!(r#"{{"command": ["get_property", "{}"]}}"#, property);
    stream.write_all(cmd.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    // Parse the JSON response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
        if let Some(data) = json.get("data") {
            // If data is a string, return it directly
            if let Some(s) = data.as_str() {
                return Ok(Some(s.to_string()));
            }
            // Otherwise, return the JSON representation
            return Ok(Some(data.to_string()));
        }
    }
    Ok(None)
}

fn get_stream_url_from_toml() -> Option<String> {
    let toml_str = fs::read_to_string("Cargo.toml").ok()?;
    let toml_value: TomlValue = toml::from_str(&toml_str).ok()?;
    toml_value
        .get("package")?
        .get("metadata")?
        .get("stream_url")?
        .as_str()
        .map(|s| s.to_string())
}

fn main() -> io::Result<()> {
    let stream_url = get_stream_url_from_toml().unwrap_or_else(|| {
        eprintln!("Stream URL not found in Cargo.toml, using default.");
        "http://stream.soundstorm-radio.com:8000".to_string()
    });
    let ipc_path = "/tmp/mpv-soundstorm.sock";

    println!("Soundstorm CLI Player");
    println!("Commands: start | pause | stop | status | exit");

    let mut mpv = None;
    let running = Arc::new(AtomicBool::new(true));
    let mut song_thread = None;

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let cmd = input.trim();

        match cmd {
            "start" => {
                if mpv.is_none() {
                    mpv = Some(start_mpv(ipc_path, &stream_url)?);
                    println!("Started playback.");

                    // Start background song info thread
                    let ipc_path = ipc_path.to_string();
                    let running = running.clone();
                    song_thread = Some(thread::spawn(move || {
                        let mut last_title = String::new();
                        while running.load(Ordering::SeqCst) {
                            if let Ok(Some(title)) = get_mpv_property(&ipc_path, "media-title") {
                                if !title.is_empty()
                                    && title != last_title
                                    && !title.contains("soundstorm-radio.com")
                                {
                                    println!("\nNow playing: {}", title);
                                    last_title = title;
                                    print!("> ");
                                    io::stdout().flush().ok();
                                }
                            }
                            thread::sleep(Duration::from_secs(2));
                        }
                    }));
                } else {
                    println!("Already playing.");
                }
            }
            "pause" => {
                let _ = send_mpv_command(ipc_path, r#""cycle", "pause""#);
                println!("Toggled pause.");
            }
            "stop" => {
                let _ = send_mpv_command(ipc_path, r#""quit""#);
                if let Some(mut child) = mpv.take() {
                    let _ = child.wait();
                }
                running.store(false, Ordering::SeqCst);
                if let Some(handle) = song_thread.take() {
                    let _ = handle.join();
                }
                println!("Stopped playback.");
            }
            "status" => {
                match get_mpv_property(ipc_path, "media-title") {
                    Ok(Some(title)) if !title.is_empty() && !title.contains("soundstorm-radio.com") => {
                        println!("Now playing: {}", title)
                    }
                    _ => {
                        // Try metadata as fallback
                        match get_mpv_property(ipc_path, "metadata") {
                            Ok(Some(meta)) if !meta.is_empty() => {
                                if let Ok(json) = serde_json::from_str::<Value>(&meta) {
                                    println!("Metadata: {:#}", json);
                                } else {
                                    println!("Metadata: {}", meta);
                                }
                            }
                            _ => println!("No song info available."),
                        }
                    }
                }
            }
            "exit" => {
                let _ = send_mpv_command(ipc_path, r#""quit""#);
                if let Some(mut child) = mpv {
                    let _ = child.wait();
                }
                running.store(false, Ordering::SeqCst);
                if let Some(handle) = song_thread {
                    let _ = handle.join();
                }
                println!("Exiting.");
                break;
            }
            _ => println!("Unknown command."),
        }
    }

    Ok(())
}
