# fab shell function - enables `f N` to cd into directories
# Add this to your .bashrc: source /path/to/fab-shell.bash

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
        # Numeric navigation - get path from fab
        local target_path
        target_path=$(command f "${args[0]}")
        
        if [[ "$edit_mode" == true ]] || [[ -f "$target_path" && ! -d "$target_path" ]]; then
            # Open in editor (forced or file)
            ${EDITOR:-micro} "$target_path"
        elif [[ -d "$target_path" ]]; then
            # cd to directory
            cd "$target_path"
        else
            echo "fab: could not open '$target_path'"
            return 1
        fi
    else
        # Normal fab invocation
        command f "$@"
    fi
}
