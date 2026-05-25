#!/run/current-system/sw/bin/bash
# Quick auto-install script

echo "🔧 Installing cfm..."

# Copy binary
mkdir -p ~/bin
cp target/release/fm ~/bin/fm

# Add PATH to current shell (for immediate testing)
export PATH="$HOME/bin:$PATH"

# Add hook for future sessions
if [[ "$SHELL" == *"zsh"* ]]; then
    echo "autoload -U add-zsh-hook" >> ~/.zshrc
    echo 'add-zsh-hook chpwd _cfm_hook' >> ~/.zshrc
    echo '_cfm_hook() { command /home/dracon/bin/fm banner "\$PWD"; }' >> ~/.zshrc
elif [[ "$SHELL" == *"bash"* ]]; then
    echo 'export PATH="\$HOME/bin:\$PATH"' >> ~/.bashrc
    echo '_cfm_hook() { command /home/dracon/bin/fm banner "\$PWD"; }' >> ~/.bashrc
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "Test it now:"
echo "  cd /home/dracon/Dev/cli-file-manager"
echo "  You should see the auto banner!"
echo ""
echo "For future sessions, source your shell config:"
echo "  source ~/.zshrc   # or source ~/.bashrc"
