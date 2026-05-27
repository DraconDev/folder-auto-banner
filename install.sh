#!/usr/bin/env bash
set -euo pipefail

echo "🔧 Installing cfm..."

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/fm"

mkdir -p "$BIN_DIR"

# Remove old binary if it exists (clean teardown)
if [ -f "$BIN_PATH" ]; then
    echo "   Removing old version..."
    rm -f "$BIN_PATH"
fi

# Copy the new binary
if [ -f "target/release/fm" ]; then
    cp target/release/fm "$BIN_PATH"
    chmod +x "$BIN_PATH"
    echo "✅ Binary installed to $BIN_PATH"
elif [ -f "$HOME/.cargo/bin/fm" ]; then
    echo "✅ Using cargo-installed binary at ~/.cargo/bin/fm"
    BIN_PATH="$HOME/.cargo/bin/fm"
else
    echo "❌ No binary found. Run 'cargo build --release' first."
    exit 1
fi

# Add PATH to shell configs (only if not already there)
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        if ! grep -q "export PATH=.*$BIN_DIR" "$rc" 2>/dev/null; then
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$rc"
            echo "✅ Added $BIN_DIR to PATH in $(basename "$rc")"
        fi
    fi
done

# Remove old hook if it exists (clean teardown)
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -f "$rc" ]; then
        sed -i '/^_cfm_hook()/,/^}/d' "$rc" 2>/dev/null || true
        sed -i '/add-zsh-hook chpwd _cfm_hook/d' "$rc" 2>/dev/null || true
        sed -i '/autoload -U add-zsh-hook/{ /^$/d; }' "$rc" 2>/dev/null || true
    fi
done

# Install hook for zsh
if [ -f "$HOME/.zshrc" ]; then
    {
        echo ''
        echo '# cfm auto-banner hook'
        echo 'autoload -U add-zsh-hook'
        echo 'add-zsh-hook chpwd _cfm_hook'
        echo "_cfm_hook() { command $BIN_PATH banner \"\$PWD\"; }"
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
echo "  source ~/.zshrc   # or: exec zsh"
echo ''
echo "Test it:"
echo "  cd $(pwd)"
echo "  You should see the banner!"
