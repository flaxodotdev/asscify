# Contributing to asscify

Thanks for hacking on ASCII art in the terminal.

## Setup

```bash
git clone https://github.com/flaxodotdev/asscify
cd asscify
cargo run -- --dir samples
```

## Checks before a PR

```bash
cargo test
cargo build --release
cargo clippy
```

## Guidelines

- Keep the UI **dark / neutral** (zinc palette) — no loud chrome.
- ASCII preview should **auto-fit** the panel and reflow on resize.
- Prefer small, focused PRs.
- Update the README if you change keys or install paths.

## Release (maintainers)

```bash
git tag v0.1.1
git push origin v0.1.1
```

The `Release` workflow builds Linux / macOS / Windows artifacts and publishes a GitHub Release with `checksums.txt`.
