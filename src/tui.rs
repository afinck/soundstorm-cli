use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};

pub fn run_tui(now_playing: Arc<Mutex<String>>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = tui_loop(&mut terminal, now_playing);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn tui_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    now_playing: Arc<Mutex<String>>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().borders(Borders::ALL).title("Soundstorm CLI Player");
            f.render_widget(block, size);

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let now_playing_guard = now_playing.lock().unwrap();
            let text = vec![
                Spans::from(Span::styled(
                    "Now playing:",
                    Style::default().fg(Color::Yellow),
                )),
                Spans::from(Span::raw(&*now_playing_guard)),
                Spans::from(""),
                Spans::from("Press 'q' to quit."),
            ];

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default());
            f.render_widget(paragraph, chunks[0]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}
