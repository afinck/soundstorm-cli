use ctrlc;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
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
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    let mut stream = UnixStream::connect(ipc_path)?;
    let json = format!(r#"{{"command": [{}]}}"#, command);
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

fn get_mpv_property(ipc_path: &str, property: &str) -> io::Result<Option<String>> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

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
    println!("Type 'help' to see available commands.");

    let mut mpv = None;
    let running = Arc::new(AtomicBool::new(true));
    let mut song_thread = None;

    // Graceful exit on Ctrl+C
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            println!("\nReceived Ctrl+C, exiting...");
        })
        .expect("Error setting Ctrl+C handler");
    }

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let cmd = input.trim().to_lowercase();

        match cmd.as_str() {
            "start" | "s" => {
                if mpv.is_none() {
                    mpv = Some(start_mpv(ipc_path, &stream_url)?);
                    println!("Started playback.");

                    for _ in 0..5 {
                        if let Ok(Some(title)) = get_mpv_property(&ipc_path, "media-title") {
                            if !title.is_empty() && !title.contains("soundstorm-radio.com") {
                                println!("Now playing: {}", title);
                                break;
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }

                    // Start background song info thread
                    let ipc_path = ipc_path.to_string();
                    let running = running.clone();
                    song_thread = Some(thread::spawn(move || {
                        let mut last_title = String::new();
                        while running.load(std::sync::atomic::Ordering::SeqCst) {
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
            "pause" | "p" => {
                let _ = send_mpv_command(ipc_path, r#""cycle", "pause""#);
                println!("Toggled pause.");
            }
            "stop" | "x" => {
                let _ = send_mpv_command(ipc_path, r#""quit""#);
                if let Some(mut child) = mpv.take() {
                    let _ = child.wait();
                }
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                if let Some(handle) = song_thread.take() {
                    let _ = handle.join();
                }
                println!("Stopped playback.");
            }
            "status" | "i" => {
                match get_mpv_property(ipc_path, "media-title") {
                    Ok(Some(title))
                        if !title.is_empty() && !title.contains("soundstorm-radio.com") =>
                    {
                        println!("Now playing: {}", title)
                    }
                    _ => {
                        // Try metadata as fallback
                        match get_mpv_property(ipc_path, "metadata") {
                            Ok(Some(meta)) if !meta.is_empty() => {
                                if let Ok(json) = serde_json::from_str::<Value>(&meta) {
                                    // Try to extract common fields
                                    if let Some(icy_title) =
                                        json.get("icy-title").and_then(|v| v.as_str())
                                    {
                                        println!("Now playing: {}", icy_title);
                                    } else if let Some(stream_title) =
                                        json.get("stream-title").and_then(|v| v.as_str())
                                    {
                                        println!("Now playing: {}", stream_title);
                                    } else if let (Some(title), Some(artist)) = (
                                        json.get("title").and_then(|v| v.as_str()),
                                        json.get("artist").and_then(|v| v.as_str()),
                                    ) {
                                        println!("Now playing: {} - {}", artist, title);
                                    } else if let Some(title) =
                                        json.get("title").and_then(|v| v.as_str())
                                    {
                                        println!("Now playing: {}", title);
                                    } else {
                                        println!(
                                            "No song metadata found. Raw metadata: {:#}",
                                            json
                                        );
                                    }
                                } else {
                                    println!("No song metadata found.");
                                }
                            }
                            _ => println!("No song info available."),
                        }
                    }
                }
            }
            "help" | "h" => {
                println!("Available commands:");
                println!("  start (s)   - Start playback");
                println!("  pause (p)   - Pause/resume playback");
                println!("  stop  (x)   - Stop playback");
                println!("  status (i)  - Show current song info");
                println!("  help  (h)   - Show this help");
                println!("  exit  (q)   - Exit the player");
            }
            "exit" | "q" => {
                let _ = send_mpv_command(ipc_path, r#""quit""#);
                if let Some(mut child) = mpv {
                    let _ = child.wait();
                }
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                if let Some(handle) = song_thread {
                    let _ = handle.join();
                }
                println!("Exiting.");
                break;
            }
            "" => continue, // Ignore empty input
            _ => println!("Unknown command. Type 'help' for a list of commands."),
        }

        // Exit if Ctrl+C was pressed
        if !running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
    }

    Ok(())
}
