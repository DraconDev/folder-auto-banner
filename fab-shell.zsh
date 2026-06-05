# f shell function - enables `f N` to cd into directories
# Add this to your .zshrc: source /path/to/fab-shell.zsh

f() {
    # Check for --edit/-e and --run/-x flags
    local edit_mode=false
    local run_mode=false
    local args=("$@")
    
    # Parse flags
    local new_args=()
    for arg in "${args[@]}"; do
        if [[ "$arg" == "--edit" || "$arg" == "-e" ]]; then
            edit_mode=true
        elif [[ "$arg" == "--run" || "$arg" == "-x" ]]; then
            run_mode=true
        else
            new_args+=("$arg")
        fi
    done
    args=("${new_args[@]}")
    
    # Check if first argument is a number
    if [[ "${args[0]}" =~ ^[0-9]+$ ]]; then
        local num="${args[0]}"
        local action="${args[1]:-}"
        
        if [[ -n "$action" ]]; then
            # f N ACTION — open item N with ACTION (e.g., f 4 krita, f 4 cat)
            command f banner "$num" "$action"
        elif [[ "$edit_mode" == true ]]; then
            # f N -e — force open in editor
            command f banner --edit "$num"
        elif [[ "$run_mode" == true ]]; then
            # f N -x — force run the file directly
            command f banner --run "$num"
        else
            # f N — default: cd for dirs, editor for files
            local target_path
            target_path=$(command f banner "$num")
            if [[ -n "$target_path" ]]; then
                cd "$target_path"
            fi
            # If target_path is empty, binary already handled the file (opened editor)
        fi
    else
        # Normal f invocation
        command f "$@"
    fi
}
