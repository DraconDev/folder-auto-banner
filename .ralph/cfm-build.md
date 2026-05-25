# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 5)

### ✅ Phase 0-4: FOUNDATION COMPLETE
### ✅ Phase 5: File Operations — DONE!
### ✅ Phase 6: Ephemeral Clipboard — DONE!
### ✅ Phase 7: Safe File Operations — DONE!
### ✅ Phase 8: Smart Piping — DONE!
### ✅ Phase 9: Directory Stats — DONE!
### ✅ Phase 10: Spatial Memory — DONE!

### Working Commands (full list):
```bash
# Banner
fm               # Rich in terminal, raw when piped
fm banner --json # JSON output

# Env
fm env           # Project aliases

# File ops
fm mv <src> <dest> [--overwrite] [--rename]
fm cp <src> <dest> [--overwrite]
fm rm <files> [-r]
fm trash <files>    # → ~/.local/share/cfm/trash/

# Clipboard
fm yank <files>
fm paste [--move-files]
fm clipboard [--clear]

# Pins
fm pin <name>
fm pins
fm jump <name>
fm unpin <name>
fm root

# Utils
fm open <files>     # Open with default app
fm do [action]      # Act on stdin paths (list, count, delete, open, cat, custom)
fm stats [--json]   # Deep directory analysis
fm trash <files>    # Move to trash
```

### Stats Output Features:
- Total size, files, directories, max depth
- File types breakdown with visual bars
- Largest files (top 10)
- Hidden files count
- Binary files detection
- Code file percentage

### ⏳ Phase 11: Session Management — NEXT
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