#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Installing cfm..."

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/fm"

mkdir -p "$BIN_DIR"

# Remove old binary if it exists (clean teardown)
if [ -f "$BIN_PATH" ]; then
    echo "   Removing old version from ~/.local/bin..."
    rm -f "$BIN_PATH"
fi

# Also remove any cargo-installed version that would take precedence
if [ -f "$HOME/.cargo/bin/fm" ]; then
    echo "   Removing old version from ~/.cargo/bin..."
    rm -f "$HOME/.cargo/bin/fm"
fi

# Copy the new binary
if [ -f "target/release/fm" ]; then
    cp target/release/fm "$BIN_PATH"
    chmod +x "$BIN_PATH"
    echo "✅ Binary installed to $BIN_PATH"
else
    echo "❌ No binary found. Run 'cargo build --release' first."
    exit 1
fi

# Copy daemon binary
if [ -f "target/release/cfmd" ]; then
    cp target/release/cfmd "$BIN_DIR/cfmd"
    chmod +x "$BIN_DIR/cfmd"
    echo "✅ Daemon binary installed to $BIN_DIR/cfmd"
fi

# Install systemd user service (optional)
if [ -d "$HOME/.config/systemd/user" ] || [ "$1" = "--with-service" ]; then
    mkdir -p "$HOME/.config/systemd/user"
    cp cfmd.service "$HOME/.config/systemd/user/cfmd.service"
    systemctl --user daemon-reload 2>/dev/null
    echo "✅ Systemd user service installed (enable with: systemctl --user enable --now cfmd)"
fi

# Add PATH to shell configs (only if not already there)
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        if ! grep -qF "$BIN_DIR" "$rc" 2>/dev/null; then
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$rc"
            echo "✅ Added $BIN_DIR to PATH in $(basename "$rc")"
        fi
    fi
done

# Clean up old hooks (all known variants: _cfm_hook, _cfm_on_directory_change, _cfm_on_startup)
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        # Remove any line containing old cfm function names or old hook registrations
        sed -i '/_cfm_hook\|_cfm_on_directory_change\|_cfm_on_startup/d' "$rc" 2>/dev/null || true
        # Remove old chpwd + precmd hook registrations
        sed -i '/add-zsh-hook \(chpwd\|precmd\) _cfm/d' "$rc" 2>/dev/null || true
        # Remove old cfm comment headers
        sed -i '/^# cfm shell integration\|^# cfm auto-banner hook/d' "$rc" 2>/dev/null || true
        # Remove orphaned function fragments (stray closing braces from partial teardowns)
        sed -i '/^    command fm banner/d' "$rc" 2>/dev/null || true
        sed -i '/command \/home\/.*\/bin\/fm banner/d' "$rc" 2>/dev/null || true
        sed -i '/^}export PATH=/d' "$rc" 2>/dev/null || true
        sed -i '/^}autoload/d' "$rc" 2>/dev/null || true
        # Remove old ~/bin PATH exports (we use ~/.local/bin now)
        sed -i '/export PATH="\$HOME\/bin:\$PATH"/d' "$rc" 2>/dev/null || true
    fi
done

# Install hook for zsh
if [ -f "$HOME/.zshrc" ]; then
    {
        echo ''
        echo '# cfm auto-banner hook'
        if ! grep -qF 'autoload -U add-zsh-hook' "$HOME/.zshrc"; then
            echo 'autoload -U add-zsh-hook'
        fi
        echo 'add-zsh-hook chpwd _cfm_hook'
        echo "_cfm_hook() { command $BIN_PATH banner \"\$PWD\"; }"
        echo '_cfm_hook  # fire on new shell/tab startup'
    } >> "$HOME/.zshrc"
    echo "✅ Added chpwd hook to ~/.zshrc"
fi

# Install hook for bash
if [ -f "$HOME/.bashrc" ]; then
    {
        echo ''
        echo '# cfm auto-banner hook'
        echo "_cfm_hook() { command $BIN_PATH banner \"\$PWD\"; }"
        echo 'PROMPT_COMMAND="_cfm_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"'
    } >> "$HOME/.bashrc"
    echo "✅ Added PROMPT_COMMAND hook to ~/.bashrc"
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
