#!/bin/bash
#
# cfm — Contextual File Manager
# Auto-install script with shell hook integration
#
# Usage: ./install.sh [bash|zsh|fish]
#
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}cfm ${GREEN}—${BLUE} Contextual File Manager${NC}"
echo "================================="
echo ""

# Detect shell
SHELL_TYPE="${1:-auto}"
case "$SHELL_TYPE" in
    bash) SHELL="bash"; SHELL_FILE=~/.bashrc;;
    zsh) SHELL="zsh"; SHELL_FILE=~/.zshrc;;
    fish) SHELL="fish"; SHELL_FILE=~/.config/fish/config;;
    *) SHELL=$([[ -f ~/.zshrc ]] && echo zsh || echo bash)
       if [[ -f ~/.zshrc ]]; then
           SHELL="zsh";
           SHELL_FILE=~/.zshrc;
       else
           SHELL="bash";
           SHELL_FILE=~/.bashrc;
       fi
       ;;
esac

echo "Detected shell: ${YELLOW}$SHELL${NC}"
echo ""

# Check if we're in the project directory
if [[ ! -f Cargo.toml ]]; then
    echo -e "${RED}Error: Not in project directory!${NC}"
    echo "Please run this from /home/dracon/Dev/cli-file-manager"
    exit 1
fi

# Check if binary exists
if [[ ! -f target/release/fm ]]; then
    echo -e "${YELLOW}Building binary...${NC}"
    cargo build --release
    if [[ $? -ne 0 ]]; then
        echo -e "${RED}Build failed!${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ Binary ready: target/release/fm${NC}"
echo ""

# Create bin directory if it doesn't exist
BIN_DIR="$HOME/bin"
mkdir -p "$BIN_DIR"

# Copy binary
cp target/release/fm "$BIN_DIR/fm"
echo -e "${GREEN}✓ Binary installed: $BIN_DIR/fm${NC}"
echo ""

# Setup shell hook
HOOK_NAME="_cfm_hook"

case "$SHELL" in
    bash)
        # Check if hook already exists
        if grep -q "^[[:space:]]*$HOOK_NAME()" "$SHELL_FILE"; then
            echo -e "${YELLOW}✓ Hook already configured in $SHELL_FILE${NC}"
            echo "  (No changes made - hook already exists)"
        else
            echo -e "${GREEN}Adding hook to $SHELL_FILE${NC}"
            echo ""
            echo "# cfm shell integration (auto banner on cd)" >> "$SHELL_FILE"
            echo "$HOOK_NAME() {" >> "$SHELL_FILE"
            echo "    command fm banner \"\$PWD\"" >> "$SHELL_FILE"
            echo "}" >> "$SHELL_FILE"
            echo ""
            echo -e "${GREEN}✓ Hook installed${NC}"
        fi
        ;;
    zsh)
        # Check if hook already exists
        if grep -q "^[[:space:]]*add-zsh-hook.*$HOOK_NAME" "$SHELL_FILE"; then
            echo -e "${YELLOW}✓ Hook already configured in $SHELL_FILE${NC}"
            echo "  (No changes made - hook already exists)"
        else
            echo -e "${GREEN}Adding hook to $SHELL_FILE${NC}"
            echo ""
            echo "# cfm shell integration (auto banner on cd)" >> "$SHELL_FILE"
            echo "autoload -U add-zsh-hook" >> "$SHELL_FILE"
            echo "add-zsh-hook chpwd $HOOK_NAME" >> "$SHELL_FILE"
            echo ""
            echo "$HOOK_NAME() {" >> "$SHELL_FILE"
            echo "    command fm banner \"\$PWD\"" >> "$SHELL_FILE"
            echo "}" >> "$SHELL_FILE"
            echo ""
            echo -e "${GREEN}✓ Hook installed${NC}"
        fi
        ;;
    fish)
        # Check if hook already exists
        if grep -q "set -x $HOOK_NAME" "$SHELL_FILE" 2>/dev/null; then
            echo -e "${YELLOW}✓ Hook already configured in $SHELL_FILE${NC}"
            echo "  (No changes made - hook already exists)"
        else
            echo -e "${GREEN}Adding hook to $SHELL_FILE${NC}"
            echo ""
            echo "# cfm shell integration (auto banner on cd)" >> "$SHELL_FILE"
            echo "function $HOOK_NAME {" >> "$SHELL_FILE"
            echo "    command fm banner \"\$PWD\"" >> "$SHELL_FILE"
            echo "end" >> "$SHELL_FILE"
            echo ""
            echo -e "${GREEN}✓ Hook installed${NC}"
        fi
        ;;
esac

echo ""
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Reload your shell: source $SHELL_FILE"
echo "2. Test: cd to different directories to see auto banner"
echo ""
echo "To uninstall:"
echo "  rm $BIN_DIR/fm"
echo "  (Remove hook from shell config manually)"
echo ""

echo -e "${GREEN}Happy dogfooding! 🐕${NC}"
