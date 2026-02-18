# Contributing to IPTV TUI

Thanks for your interest in contributing! This project is open to contributions of all kinds — bug reports, feature requests, documentation improvements, and code.

## Reporting Issues

- **Bug reports** — include steps to reproduce, expected vs actual behavior, and your terminal/OS info
- **Feature requests** — describe the use case and why it would be valuable
- **Questions** — open a discussion or issue, happy to help

## Development Setup

```bash
# Clone and build
git clone https://github.com/nschnarr/iptv-tui-player.git
cd iptv-tui-player
cargo build

# Run with the free iptv-org playlist for testing
cargo run -- --url "https://iptv-org.github.io/iptv/index.m3u"

# Run clippy (must pass with no warnings)
cargo clippy -- -D warnings
```

### Requirements

- Rust 1.75+ (stable)
- VLC or mpv for playback testing (optional)
- ffmpeg for HDHR buffer mode testing (optional)

## Pull Request Process

1. **Fork** the repo and create a feature branch from `main`
2. **Keep PRs focused** — one feature or fix per PR
3. **Follow existing code patterns** — look at how similar features are implemented
4. **Ensure it compiles** — `cargo build` must succeed
5. **Run clippy** — `cargo clippy -- -D warnings` must pass clean
6. **Test with real data** — try your changes with a large playlist
7. **Write a clear commit message** — use conventional format:
   - `feat: add channel sorting options`
   - `fix: correct EPG timezone offset`
   - `refactor: extract theme system into module`
   - `docs: update README with new keybindings`

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- No `unsafe` unless absolutely necessary and well-documented
- Keep functions focused — prefer small, composable functions
- Use the theme system for all UI colors (no hardcoded `Color::` in `ui.rs`)
- Error handling: use `anyhow` for application errors, `thiserror` for library-style errors

## Architecture Overview

The app follows a straightforward structure:

- **`app.rs`** — all application state lives in the `App` struct
- **`ui.rs`** — pure rendering functions that take `&App` and draw to the frame
- **`main.rs`** — event loop, key handling, background thread management
- **`theme.rs`** — semantic color system with built-in theme presets

New features typically involve:
1. Adding state to `App` in `app.rs`
2. Adding a draw function in `ui.rs`
3. Adding key handling in `main.rs`
4. Adding an `AppMode` variant if it's a new overlay/panel

## Areas for Contribution

Some areas that could use help (also tracked in [issues](https://github.com/nschnarr/iptv-tui-player/issues)):

- **Multi-provider support** — merge multiple M3U sources into one view
- **TMDB metadata** — enrich series/movie channels with poster art, ratings, descriptions
- **Watch history** — track what you've watched, resume where you left off
- **Smart collections** — auto-group channels by content type, language, quality
- **Additional themes** — add your favorite color scheme to `theme.rs`
- **Platform packaging** — Homebrew formula, AUR package, Nix flake, etc.
- **CI/CD** — cross-platform release builds (Linux, macOS, Windows)
- **Tests** — unit tests for parser, EPG, series tree builder

## Adding a Theme

Adding a new theme is one of the easiest contributions:

1. Open `src/theme.rs`
2. Add a new function following the pattern of existing themes (e.g., `catppuccin()`)
3. Add the theme name to `THEME_NAMES`
4. Add the match arm in `by_name()`
5. Fill in all 13 color roles from your chosen palette
6. Test it: `cargo run -- --url "https://iptv-org.github.io/iptv/index.m3u"` then press `t` to cycle

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
