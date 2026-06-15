# f — Installation Guide

## Recommended install

Build and install the binary, shell wrappers, and daemon hooks from the repository:

```bash
./install.sh
```

The script installs:

- `f` and `fabd` into `~/.local/bin`
- shell wrappers for `f N` navigation into `~/.local/bin`
- source lines in `~/.zshrc` and/or `~/.bashrc`
- the user systemd service for the background daemon, if available

Start a new shell, or activate the shell wrappers immediately:

```bash
source ~/.local/bin/fab-shell.zsh   # zsh
source ~/.local/bin/fab-shell.bash  # bash
```

## Cargo install

Install the latest released version from crates.io:

```bash
cargo install folder-auto-banner
```

Then install the shell wrappers for `f N` navigation:

```bash
f install
```

Start a new shell, or source the wrappers immediately as shown above.

## Manual build from this checkout

```bash
cargo build --release
./install.sh
f install
```

## Daemon management

The background daemon is managed through `f daemon`:

```bash
f daemon status   # show daemon state
f daemon restart  # restart daemon
f daemon stop     # stop daemon
f daemon start    # start daemon
```

If the banner ever looks stale, restart the daemon and give the background size refresh a moment:

```bash
f daemon restart
f -b ~/Downloads
sleep 35
f -b ~/Downloads
```

The first fast view of a large directory may show temporary `4.0k` placeholders while the daemon refreshes directory sizes in the background. After the refresh completes, subsequent views should use populated sizes from cache.

## Testing

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```
