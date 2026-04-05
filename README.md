# Lobber

An artillery game inspired by [Scorched Earth](https://en.wikipedia.org/wiki/Scorched_Earth_(video_game)) (1991), built in Rust with [macroquad](https://github.com/not-fl3/macroquad).

<video src="demo.mp4" autoplay loop muted playsinline width="100%"></video>

## Features

- Destructible terrain with procedural generation
- Two ammo types: **Cannonball** (precision, heavy damage) and **Explosive** (splash radius)
- Wind that changes each turn, visualized with animated clouds
- WoT-style RNG: damage spread, accuracy deviation, critical hits
- AI opponent with adjustable difficulty
- Trajectory spread cone showing aim uncertainty
- Tank movement during aiming phase
- Physics simulation with gravity, wind, and drag

## Controls

| Key | Action |
|-----|--------|
| `h`/`Left` `l`/`Right` | Adjust angle |
| `k`/`Up` `j`/`Down` | Adjust power |
| `a` / `d` | Move tank |
| `Tab` | Switch ammo |
| `Space` | Fire |
| `Esc` | Quit |

## Building

Requires Rust 2024 edition.

**Linux** (needs X11/GL dev libraries):
```bash
sudo apt-get install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
cargo run --release
```

**macOS / Windows**:
```bash
cargo run --release
```

## Downloads

Pre-built binaries are available on the [Releases](../../releases) page for:
- Linux (x86_64)
- macOS (Intel + Apple Silicon)
- Windows (x86_64)

## Author

Miles Granger

## License

MIT
