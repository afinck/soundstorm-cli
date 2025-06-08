# Soundstorm CLI Radio Player

A terminal-based radio player for [Soundstorm Radio](https://soundstorm-radio.com), built in Rust with a live-updating TUI.

## Features

- Plays Soundstorm Radio stream (or any custom stream URL) using `mpv`
- Live-updating "Now Playing" info in the terminal
- Simple, keyboard-driven controls
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

2. **Build and run with the default stream:**
   ```sh
   cargo run
   ```

3. **Or run with a custom stream URL:**
   ```sh
   cargo run -- http://your.custom.stream/url
   ```
   Or, if you built a release binary:
   ```sh
   ./target/release/soundstorm-cli http://your.custom.stream/url
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

MIT OR Apache-2.0

---

Made with ❤️ for Soundstorm Radio fans.