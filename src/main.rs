use std::env;
use std::io;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

const DEFAULT_STREAM_URL: &str = "http://stream.soundstorm-radio.com:8000";

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

fn run_tui(
    now_playing: Arc<Mutex<String>>,
    stream_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ipc_path = "/tmp/mpv-soundstorm.sock";

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut running = true;
    let mut mpv: Option<Child> = None;

    // Main loop
    while running {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let title = Paragraph::new("Soundstorm Radio CLI").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(title, chunks[0]);

            let now_playing_text = now_playing.lock().unwrap().clone();
            let status_par = Paragraph::new(format!("Now playing: {}", now_playing_text))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_par, chunks[1]);

            let help = Paragraph::new(Line::from(vec![
                Span::styled("[S]tart ", Style::default().fg(Color::Green)),
                Span::raw("[P]ause "),
                Span::styled("[X] Stop ", Style::default().fg(Color::Red)),
                Span::raw("[Q]uit"),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        running = false;
                        if let Some(mut child) = mpv.take() {
                            let _ = send_mpv_command(ipc_path, r#""quit""#);
                            let _ = child.wait();
                        }
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if mpv.is_none() {
                            mpv = Some(start_mpv(ipc_path, stream_url)?);
                            for _ in 0..5 {
                                if let Ok(Some(title)) = get_mpv_property(ipc_path, "media-title") {
                                    if !title.is_empty() && !title.contains("soundstorm-radio.com")
                                    {
                                        let mut np = now_playing.lock().unwrap();
                                        *np = title;
                                        break;
                                    }
                                }
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        let _ = send_mpv_command(ipc_path, r#""cycle", "pause""#);
                    }
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        if let Some(mut child) = mpv.take() {
                            let _ = send_mpv_command(ipc_path, r#""quit""#);
                            let _ = child.wait();
                        }
                        let mut np = now_playing.lock().unwrap();
                        *np = "No song info yet".to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let stream_url = if args.len() > 1 {
        &args[1]
    } else {
        DEFAULT_STREAM_URL
    };

    let ipc_path = "/tmp/mpv-soundstorm.sock";
    let now_playing = Arc::new(Mutex::new("No song info yet".to_string()));

    let now_playing_clone = Arc::clone(&now_playing);
    let ipc_path_string = ipc_path.to_string();
    thread::spawn(move || {
        let mut last_title = String::new();
        loop {
            // Try to get media-title, fallback to metadata
            let title = match get_mpv_property(&ipc_path_string, "media-title") {
                Ok(Some(title)) if !title.is_empty() && !title.contains("soundstorm-radio.com") => {
                    title
                }
                Ok(_) => last_title.clone(),  // No new info, keep last
                Err(_) => last_title.clone(), // mpv not running, keep last
            };
            // ... (rest of your metadata fallback logic here, unchanged) ...
            if title != last_title && !title.is_empty() {
                let mut np = now_playing_clone.lock().unwrap();
                *np = title.clone();
                last_title = title;
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    // Run the TUI
    run_tui(now_playing, stream_url)?;
    Ok(())
}
