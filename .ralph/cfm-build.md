# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 4)

### ✅ Phase 0-4: FOUNDATION COMPLETE
- Banner, env, state types, TTY detection all working

### ✅ Phase 5: File Operations — DONE!
```bash
fm mv <src> <dest> [--overwrite] [--rename]   # Move with collision detection
fm cp <src> <dest> [--overwrite]               # Copy with collision detection
fm rm <files> [-r]                              # Safe remove (no protected paths)
fm trash <files>                                # Move to ~/.local/share/cfm/trash/
```

**Working features:**
- Collision detection (skip/overwrite/rename)
- Cross-device move support
- Protected path safety
- Trash with manifest for recovery

### ✅ Phase 6: Ephemeral Clipboard — DONE!
```bash
fm yank <files>    # Copy to clipboard
fm paste           # Paste from clipboard  
fm paste --move-files  # Move from clipboard
fm clipboard       # Show clipboard
fm clipboard --clear  # Clear clipboard
```

### ✅ Phase 10: Spatial Memory — DONE!
```bash
fm pin <name>       # Bookmark directory
fm pins            # List pins
fm jump <name>     # Print cd command
fm unpin <name>    # Remove pin
fm root            # Jump to git root
```

### ⏳ Phase 7: Safe File Operations — NEXT
- [ ] `fm open` — open with default app
- [ ] `fm do` — stdin pipe destination

### ⏳ Phase 8: Smart Piping
- [ ] `fm do` — stdin pipe destination

### ⏳ Phase 9: Stats
- [ ] `fm stats` — deep synthesis

### ⏳ Phase 11: Session Management
- [ ] `fm save-session`
- [ ] `fm load-session`
- [ ] `fm sessions`
- [ ] `fm delete-session`

### ⏳ Phase 12-18: Future phases
- Diff, completions, config, polish, tests, docs, packaging

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README