# TimeTracker (Titrax GTK4 Rewrite)

A minimal click-to-track time tracker for GNOME/GTK4, written in Rust.

This is a modern rewrite of the original **Titrax** — a time-tracking program written by Harald Tveit Alvestrand in 1996 using the X11 Athena widget toolkit.
Full credit and thanks to the original author.
Original program and documentation: [https://www.alvestrand.no/titrax/](https://www.alvestrand.no/titrax/)
License terms for this repository are documented in `LICENSE` and are based on
the original Titrax distribution terms.

## Build

If no rust from before:
```bash
rustup toolchain list
rustup install stable
```

```bash
cargo build --release
./target/release/titrax-rs
```

If you start it from a terminal and want the prompt back immediately, use:

```bash
./target/release/titrax-rs &
```

If you start it from an application icon or desktop launcher, no `&` is needed.

The repo also includes a legacy-style app icon in `share/icons/hicolor/64x64/apps/titrax-rs.png`
and a desktop entry in `share/applications/titrax-rs.desktop` so the launcher can show the
old Titrax icon on modern desktops.

For a per-user install without touching system directories, run:

```bash
./scripts/install-user.sh
```

This installs the binary to `~/.local/bin`, the icon to `~/.local/share/icons/`, and a
desktop entry to `~/.local/share/applications/`. It prefers `target/debug/titrax-rs` when
present, so it follows your current development build; otherwise it falls back to release
or builds debug on demand.

To build a user-mode `.deb` package (no root needed) from the **release** binary:

```bash
cargo build --release
./scripts/build-user-deb.sh --release
```

The resulting package is written to `dist/titrax-rs-user_<version>_<arch>.deb`.
This is the file suitable for distribution and download.

To build and install it directly for the current user (`~/.local`) in one step:

```bash
cargo build --release
./scripts/build-user-deb.sh --release --install
```

### Install from a pre-built `.deb` (quickest)

Pre-built packages are published on the
[Releases page](https://github.com/zerblatt007/titrax-rs/releases).
This one-liner downloads the latest release and installs it into `~/.local` — no root needed:

```bash
curl -sL $(curl -s https://api.github.com/repos/zerblatt007/titrax-rs/releases/latest \
  | grep -o 'https://[^"]*amd64\.deb') -o /tmp/titrax-rs.deb \
  && dpkg-deb -x /tmp/titrax-rs.deb ~/.local
```

After installation, make sure `~/.local/bin` is on your `PATH`. Then run:

```bash
titrax-rs &
```

To uninstall, run the uninstall script from a cloned copy of the repo:

```bash
./scripts/uninstall-user.sh
```

Note: `dpkg remove` does not apply here — the package is extracted directly into your home
directory and is not registered in the system dpkg database.

## Requirements

### Building the application

- Rust >= 1.70 (install via [rustup](https://rustup.rs/))
- GTK4 >= 4.6 — development headers: `sudo apt install libgtk-4-dev`
- `pkg-config`: `sudo apt install pkg-config`

**GTK4 crate version used:** `gtk4 = "0.7"` (tested against system GTK4 4.8.3)

### Building the `.deb` package

In addition to the above, you need:

- `dpkg-deb` and `dpkg-architecture` (part of `dpkg-dev`): `sudo apt install dpkg-dev`

## Data

- Day files: `~/.TimeTracker/YYYY-MM-DD` — one file per day, format: `HH:MM ProjectName`
- Config: `~/.config/titrax-rs/config.toml`
- Respects `TIMETRACKDIR` and `TIMEXDIR` environment variables for data directory override.

## Usage

- **Left-click** a project to start tracking it
- **Right-click** for context menu: transfer minutes, edit time, delete
- **Pause** button to pause/resume tracking the active project
- **Add** button to add a new project
- **Sort A-Å** for Norwegian alphabetical sort (Æ, Ø, Å sort after Z)
- **Ctrl++** / **Ctrl+-** to adjust font size; **Ctrl+0** to reset to 12pt

## Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `scripts/bump-version.sh` | Bump Cargo.toml version | `./scripts/bump-version.sh [patch\|minor\|major]` |
| `scripts/install-user.sh` | Install titrax-rs for the current user | `./scripts/install-user.sh [path-to-binary]` |
| `scripts/build-user-deb.sh` | Build user-mode .deb (optional local install) | `./scripts/build-user-deb.sh [--install] [path-to-binary]` |
| `scripts/uninstall-user.sh` | Remove user-mode install from ~/.local | `./scripts/uninstall-user.sh [--dry-run]` |
