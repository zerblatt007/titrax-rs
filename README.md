# TimeTracker (Titrax GTK4 Rewrite)

A minimal click-to-track time tracker for GNOME/GTK4, written in Rust. Rewrite of the original 1996 Athena-widget C program.

## Build

```bash
cargo build --release
./target/release/titrax
```

## Requirements

- GTK4 >= 4.6 (`libgtk-4-dev`)
- Rust >= 1.70
- `pkg-config`

**GTK4 crate version used:** `gtk4 = "0.7"` (system GTK4 is 4.8.3)

## Data

- Day files: `~/.TimeTracker/YYYY-MM-DD` — one file per day, format: `HH:MM ProjectName`
- Config: `~/.config/titrax/config.toml`
- Respects `TIMETRACKDIR` and `TIMEXDIR` environment variables for data directory override.

## Usage

- **Left-click** a project to start tracking it (click again to pause)
- **Right-click** for context menu: mark, transfer minutes, edit time, delete
- **Add** button to add a new project
- **Sort A-Å** for Norwegian alphabetical sort (Æ, Ø, Å sort after Z)
- **Pause** button to stop tracking
- **Ctrl++** / **Ctrl+-** to adjust font size; **Ctrl+0** to reset to 12pt

## Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `scripts/bump-version.sh` | Bump Cargo.toml version | `./scripts/bump-version.sh [patch\|minor\|major]` |
