# Soundstorm CLI Radio Player

A terminal-based radio player for [Soundstorm Radio](https://soundstorm-radio.com), built in Rust with a live-updating TUI.

## Features

- Plays Soundstorm Radio stream using `mpv`
- Live-updating "Now Playing" info in the terminal
- Simple, keyboard-driven controls
- Reads stream URL from `Cargo.toml` (customizable)
- Clean, cross-platform TUI using [ratatui](https://crates.io/crates/ratatui) and [crossterm](https://crates.io/crates/crossterm)

## Controls

- **q** — Quit
- **s** — Start playback
- **p** — Pause/resume playback
- **x** — Stop playback

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [mpv](https://mpv.io/) media player installed and available in your `$PATH`

## Usage

1. **Clone the repository:**
   ```sh
   git clone https://gitlab.com/<your-username>/<your-repo>.git
   cd <your-repo>
   ```

2. **(Optional) Edit the stream URL:**

   In your `Cargo.toml`, set the stream URL under `[package.metadata]`:
   ```toml
   [package.metadata]
   stream_url = "http://stream.soundstorm-radio.com:8000"
   ```

3. **Build and run:**
   ```sh
   cargo run
   ```

4. **Enjoy the music!**  
   Use the keyboard controls shown in the TUI.

## Docker

You can also run the app in a container (see `Dockerfile`):

```sh
docker build -t soundstorm-cli .
docker run --rm -it --device /dev/snd soundstorm-cli
```

## License

MIT

---

Made with ❤️ for Soundstorm Radio fans.