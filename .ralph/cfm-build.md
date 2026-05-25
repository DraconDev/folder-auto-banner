# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool, following the todo.md phases.

## Progress (Iteration 3)

### ✅ Phase 0: Project Foundation — DONE
### ✅ Phase 1: Core Architecture — DONE
### ✅ Phase 2: The Banner — DONE (working!)
### ✅ Phase 3: Shell Integration — STUBS DONE
### ✅ Phase 4: Context-Aware Environment — DONE

### ✅ Phase 6: Ephemeral Clipboard (`fm yank` / `fm paste` / `fm clipboard`) — DONE!
**Working commands:**
```bash
fm yank Cargo.toml todo.md   # → Copies to clipboard
fm clipboard                 # → Shows clipboard contents
fm paste                    # → Copies from clipboard
fm paste --move-files       # → Moves files (clears entry)
fm clipboard --clear        # → Clears clipboard
```

### ✅ Phase 10: Spatial Memory (`fm pin` / `fm jump` / `fm root`) — DONE!
**Working commands:**
```bash
fm pin dev                  # → Pin current dir as "dev"
fm pin src                  # → Pin another dir
fm pins                     # → List all pins
fm jump dev                 # → Print cd command
fm unpin dev               # → Remove pin
fm root                    # → Print cd to git root
```

### ⏳ Phase 5: File Operations (`fm mv` / `fm cp` / `fm rm` / `fm trash`) — NEXT
- Collision detection (--overwrite, --rename, --skip)

### ⏳ Phase 7: Safe File Operations (`fm open`)
- Open files with default app

## Status

Core functionality is working:
- Banner with TTY detection (rich/JSON/raw modes)
- Clipboard (yank/paste/clipboard)
- Pins (pin/jump/pins/unpin/root)

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README