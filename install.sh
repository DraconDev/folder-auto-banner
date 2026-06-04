#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Installing fab..."

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/f"
DAEMON_BIN="$BIN_DIR/fabd"
SOCKET_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/cfm"
SOCKET_PATH="$SOCKET_DIR/fabd.sock"

mkdir -p "$BIN_DIR"

# --- Kill running daemon and clean up socket ---
if [ -f "$DAEMON_BIN" ] && pgrep -x fabd > /dev/null 2>&1; then
    echo "   Stopping running daemon..."
    
    # First try to stop via systemd if running as a service
    if systemctl --user is-active fabd.service > /dev/null 2>&1; then
        echo "   Stopping systemd service..."
        systemctl --user stop fabd.service 2>/dev/null || true
        sleep 1
    fi
    
    # Send shutdown signal via socket if possible
    "$BIN_DIR/f" daemon stop 2>/dev/null || true
    sleep 1
    
    # Force kill any remaining processes
    pkill -9 -x fabd 2>/dev/null || true
    sleep 1
    
    # Verify daemon is dead
    if pgrep -x fabd > /dev/null 2>&1; then
        echo "   ⚠️  Warning: daemon still running, waiting..."
        sleep 2
        pkill -9 -x fabd 2>/dev/null || true
        sleep 1
    fi
fi
# Always clean up stale socket
if [ -S "$SOCKET_PATH" ]; then
    rm -f "$SOCKET_PATH"
    echo "   Removed stale socket"
fi

# --- Remove old binaries from all known locations ---
for loc in "$BIN_DIR/f" "$HOME/.cargo/bin/f" "$HOME/bin/f" "/usr/local/bin/f"; do
    if [ -f "$loc" ]; then
        echo "   Removing old f from $loc..."
        rm -f "$loc"
    fi
done

for loc in "$BIN_DIR/fabd" "$HOME/.cargo/bin/fabd" "$HOME/bin/fabd" "/usr/local/bin/fabd"; do
    if [ -f "$loc" ]; then
        echo "   Removing old fabd from $loc..."
        rm -f "$loc"
    fi
done

# --- Always build release to ensure latest version ---
echo "   Building release binaries..."
cargo build --release

# --- Copy new binaries ---
cp target/release/f "$BIN_PATH"
chmod +x "$BIN_PATH"
echo "✅ f installed to $BIN_PATH"

cp target/release/fabd "$DAEMON_BIN"
chmod +x "$DAEMON_BIN"
echo "✅ fabd installed to $DAEMON_BIN"

# --- Install systemd user service (if fabd.service exists) ---
if [ -f "fabd.service" ]; then
    mkdir -p "$HOME/.config/systemd/user"
    cp fabd.service "$HOME/.config/systemd/user/fabd.service"
    systemctl --user daemon-reload 2>/dev/null || true
    echo "✅ Systemd user service installed"
    echo "   Enable with: systemctl --user enable --now fabd"
fi

# --- Add PATH to shell configs (only if not already there) ---
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        if ! grep -qF "$BIN_DIR" "$rc" 2>/dev/null; then
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$rc"
            echo "✅ Added $BIN_DIR to PATH in $(basename "$rc")"
        fi
    fi
done

# --- Clean up old hooks (all known variants) ---
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        # Remove old fab function names and hook registrations
        sed -i '/_fab_hook\|_fab_on_directory_change\|_fab_on_startup/d' "$rc" 2>/dev/null || true
        sed -i '/add-zsh-hook \(chpwd\|precmd\) _cfm/d' "$rc" 2>/dev/null || true
        sed -i '/^# fab shell integration\|^# fab auto-banner hook/d' "$rc" 2>/dev/null || true
        # Remove orphaned function fragments from partial teardowns
        sed -i '/^    command f banner/d' "$rc" 2>/dev/null || true
        sed -i '/command \/home\/.*\/bin\/f banner/d' "$rc" 2>/dev/null || true
        sed -i '/^}export PATH=/d' "$rc" 2>/dev/null || true
        sed -i '/^}autoload/d' "$rc" 2>/dev/null || true
        # Remove old ~/bin PATH exports (we use ~/.local/bin now)
        sed -i '/export PATH="\$HOME\/bin:\$PATH"/d' "$rc" 2>/dev/null || true
    fi
done

# --- Install hook for zsh ---
if [ -f "$HOME/.zshrc" ]; then
    {
        echo ''
        echo '# fab auto-banner hook'
        if ! grep -qF 'autoload -U add-zsh-hook' "$HOME/.zshrc"; then
            echo 'autoload -U add-zsh-hook'
        fi
        echo 'add-zsh-hook chpwd _fab_hook'
        echo "_fab_hook() { command $BIN_PATH banner \"\$PWD\"; }"
        echo '_fab_hook  # fire on new shell/tab startup'
    } >> "$HOME/.zshrc"
    echo "✅ Added chpwd hook to ~/.zshrc"
fi

# --- Install hook for bash ---
if [ -f "$HOME/.bashrc" ]; then
    {
        echo ''
        echo '# fab auto-banner hook'
        echo "_fab_hook() { command $BIN_PATH banner \"\$PWD\"; }"
        echo 'PROMPT_COMMAND="_fab_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"'
    } >> "$HOME/.bashrc"
    echo "✅ Added PROMPT_COMMAND hook to ~/.bashrc"
fi

# --- Start daemon ---
echo "   Starting daemon..."
"$DAEMON_BIN" &
sleep 1
if pgrep -x fabd > /dev/null 2>&1; then
    echo "✅ Daemon started"
else
    echo "⚠️  Daemon failed to start (it will auto-start on first banner)"
fi

echo ''
echo "✅ Installation complete!"
echo ''
echo "Reload your shell:"
echo "  exec zsh   # or: source ~/.bashrc"
echo ''
echo "Test it:"
echo "  cd $(pwd)"
echo "  You should see the banner!"
