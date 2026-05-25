#!/run/current-system/sw/bin/bash
# Auto-fix installation script

echo "🔧 Fixing cfm installation..."

# 1. Copy binary to bin directory
mkdir -p ~/bin
cp target/release/fm ~/bin/fm 2>/dev/null || cp /home/dracon/Dev/cli-file-manager/target/release/fm ~/bin/fm

# 2. Set PATH (add to current shell for immediate effect)
export PATH="$HOME/bin:$PATH"
echo "✅ Added ~/bin to PATH"

# 3. Add hook to current shell
if [[ "$SHELL" == *"zsh"* ]]; then
    source ~/.zshrc 2>/dev/null || true
elif [[ "$SHELL" == *"bash"* ]]; then
    source ~/.bashrc 2>/dev/null || true
fi

# 4. Update hook in current shell
if ! type _cfm_hook >/dev/null 2>&1; then
    if [[ "$SHELL" == *"zsh"* ]]; then
        echo "autoload -U add-zsh-hook" >> ~/.zshrc
        echo 'add-zsh-hook chpwd _cfm_hook' >> ~/.zshrc
        echo '_cfm_hook() { command /home/dracon/bin/fm banner "$PWD"; }' >> ~/.zshrc
        echo "✅ Added hook to ~/.zshrc"
    fi
    if [[ "$SHELL" == *"bash"* ]]; then
        echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
        echo '_cfm_hook() { command /home/dracon/bin/fm banner "$PWD"; }' >> ~/.bashrc
        echo "✅ Added hook to ~/.bashrc"
    fi
    
    # Reload current shell
    if [[ "$SHELL" == *"zsh"* ]]; then
        source ~/.zshrc
        echo "✅ Sourced ~/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        source ~/.bashrc
        echo "✅ Sourced ~/.bashrc"
    fi
fi

echo ""
echo "🎉 Now test it:"
echo "cd /home/dracon/Dev/cli-file-manager"
echo "You should see the banner automatically!"
