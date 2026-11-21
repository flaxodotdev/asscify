# asscify

**Beautiful terminal gallery that turns images into ASCII art.**

Dark TUI. Auto-fits your panel. Live preview. Showcase mode. Save with one key.

```
 ▍ asscify  image → ASCII  ·  dark terminal gallery
┌─ images ─────┐┌─ preview [blocks · 140×42 · zoom 100%] ──────────┐
│› orb.png     ││              ░▒▓████▓▒░                          │
│  gradient.png││           ▒▓████████████▓▒                       │
└──────────────┘└──────────────────────────────────────────────────┘
 [↑↓] nav  [+/-] zoom  [w] SAVE  [?] help  [q] quit
```

[![CI](https://github.com/flaxodotdev/asscify/actions/workflows/ci.yml/badge.svg)](https://github.com/flaxodotdev/asscify/actions/workflows/ci.yml)
[![Release](https://github.com/flaxodotdev/asscify/actions/workflows/release.yml/badge.svg)](https://github.com/flaxodotdev/asscify/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Install

### One-liner (recommended)

**macOS / Linux**

```bash
curl -fsSL https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.sh | bash
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/flaxodotdev/asscify/master/install.ps1 | iex
```

Binaries ship for:

| OS | Arch |
|----|------|
| Linux | `x86_64`, `aarch64` |
| macOS | `x86_64` (Intel), `aarch64` (Apple Silicon) |
| Windows | `x86_64` |

### Cargo

```bash
cargo install --git https://github.com/flaxodotdev/asscify
```

Or from this repo:

```bash
cargo install --path .
```

### Manual download

Grab a build from [Releases](https://github.com/flaxodotdev/asscify/releases), unpack, and put `asscify` on your `PATH`.

## Usage

```bash
asscify                  # TUI in the current directory
asscify --dir ~/Pictures
asscify showcase         # procedural orb, no images needed
asscify convert photo.png -w 100 -s blocks
```

### Keys

| Key | Action |
|-----|--------|
| `↑` `↓` / `j` `k` | Move through images |
| `+` `-` | Zoom (ASCII auto-fits the panel) |
| `0` | Reset zoom to fill the panel |
| `[` `]` | Cycle styles |
| `c` | Toggle truecolor |
| `i` | Invert luminance |
| `s` / `Enter` | Fullscreen showcase |
| `w` | **Save** `.ascii.txt` in the current folder |
| `r` | Rescan directory |
| `?` | Help overlay |
| `q` / `Esc` | Back / quit |

**Save:** press **`w`**. Browse mode writes `<image-stem>.ascii.txt`; showcase writes `showcase.ascii.txt`.

### Styles

`classic` · `blocks` · `detailed` · `minimal` · `binary`

## Build from source

```bash
git clone https://github.com/flaxodotdev/asscify
cd asscify
cargo build --release
./target/release/asscify --dir samples
```

Requirements: Rust 1.75+, a truecolor terminal (Ghostty, Kitty, WezTerm, iTerm2, Windows Terminal, …).

## Contributing

PRs welcome. Keep the TUI dark and the ASCII auto-fit snug.

```bash
cargo test
cargo build --release
```

## License

[MIT](LICENSE) © flaxodotdev
