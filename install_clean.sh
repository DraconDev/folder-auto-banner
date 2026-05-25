#!/run/current-system/sw/bin/bash
# Clean install script for zsh

echo "🔧 Installing cfm for zsh..."

# Copy binary
mkdir -p ~/bin
cp target/release/fm ~/bin/fm

# Add PATH to ~/.zshrc (at the end, not duplicate)
if ! grep -q 'export PATH=.*\$HOME/bin' ~/.zshrc; then
    echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
    echo "✅ Added PATH to ~/.zshrc"
fi

# Add hook to ~/.zshrc (at the end, remove duplicates)
if grep -q '_cfm_hook()' ~/.zshrc; then
    # Remove old hooks
    sed -i '/^_cfm_hook()/d' ~/.zshrc
    sed -i '/^add-zsh-hook chpwd _cfm_hook/d' ~/.zshrc
    echo "✅ Added hook to ~/.zshrc"
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "To test, reload zsh:"
echo "  exit"
echo "  zsh"
echo "  cd /home/dracon/Dev/cli-file-manager"
echo "  You should see the auto banner!"
