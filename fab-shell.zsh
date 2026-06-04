# fab shell function - enables `f N` to cd into directories
# Add this to your .zshrc: source /path/to/fab-shell.zsh

f() {
    # Check for --edit flag
    local edit_mode=false
    local args=("$@")
    
    # Parse --edit or -e flag
    local new_args=()
    for arg in "${args[@]}"; do
        if [[ "$arg" == "--edit" || "$arg" == "-e" ]]; then
            edit_mode=true
        else
            new_args+=("$arg")
        fi
    done
    args=("${new_args[@]}")
    
    # Check if first argument is a number
    if [[ "${args[0]}" =~ ^[0-9]+$ ]]; then
        # Numeric navigation - binary handles:
        # - directories: prints path (for cd below)
        # - files: opens editor directly (no output)
        local target_path
        target_path=$(command f "${args[0]}")
        
        if [[ "$edit_mode" == true ]] && [[ -n "$target_path" ]]; then
            # Force edit mode on a directory path
            ${EDITOR:-micro} "$target_path"
        elif [[ -n "$target_path" ]]; then
            # Directory - cd into it
            cd "$target_path"
        fi
        # If target_path is empty, binary already handled the file (opened editor)
    else
        # Normal fab invocation
        command f "$@"
    fi
}
