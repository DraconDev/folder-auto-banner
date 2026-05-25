#!/run/current-system/sw/bin/bash
# cfm — Auto Install Script

echo "🔧 Installing cfm..."

# Copy binary
mkdir -p ~/bin
cp target/release/fm ~/bin/fm

# Add PATH to ~/.zshrc (only if not already there)
if ! grep -q 'export PATH=.*\$HOME/bin' ~/.zshrc 2>/dev/null; then
    echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
    echo "✅ Added PATH to ~/.zshrc"
fi

# Add hook to ~/.zshrc (only if not already there)
if ! grep -q '_cfm_hook() {' ~/.zshrc 2>/dev/null; then
    echo 'autoload -U add-zsh-hook' >> ~/.zshrc
    echo 'add-zsh-hook chpwd _cfm_hook' >> ~/.zshrc
    echo '_cfm_hook() { command fm banner "\$PWD"; }' >> ~/.zshrc
    echo "✅ Added hook to ~/.zshrc"
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "Test it:"
echo "  source ~/.zshrc"
echo "  cd /home/dracon/Dev/cli-file-manager"
echo "  You should see the auto banner!"
