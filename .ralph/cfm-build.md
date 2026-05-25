# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool, following the todo.md phases.

## Progress (Iteration 3)

### ✅ Phase 0: Project Foundation — DONE
### ✅ Phase 1: Core Architecture — DONE
### ✅ Phase 2: The Banner — DONE (working!)
### ✅ Phase 3: Shell Integration — STUBS DONE
### ✅ Phase 4: Context-Aware Environment — DONE

### ✅ Phase 6: Ephemeral Clipboard (`fm yank` / `fm paste` / `fm clipboard`) — DONE!
- `fm yank <paths>`: Resolves paths, saves to clipboard.json
- `fm paste`: Copies files from clipboard to current dir
- `fm paste --move-files`: Moves files instead of copying
- `fm clipboard`: Shows clipboard contents
- `fm clipboard --clear`: Clears clipboard
- State persistence via ClipboardState in ~/.local/share/cfm/clipboard.json

**Working commands:**
```bash
fm yank Cargo.toml todo.md   # → Copies to clipboard
fm clipboard                 # → Shows clipboard contents
fm paste                    # → Copies from clipboard (skip existing)
fm paste --move-files       # → Moves from clipboard (clears entry)
fm clipboard --clear        # → Clears clipboard
```

### ⏳ Phase 5: File Operations (`fm mv`) — NEXT
- Split context dashboard for move operations
- Collision detection (--overwrite, --rename, --skip)

### ⏳ Phase 7: Safe File Operations (`fm rm` / `fm trash` / `fm open`)
- Confirmation, dry-run, piped input support

### ⏳ Phase 10: Spatial Memory (`fm pin` / `fm jump` / `fm root`)
- Bookmark system with shell wrapper

## Status

Core banner and clipboard are working. Next: pin/jump/root, then file operations.

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README