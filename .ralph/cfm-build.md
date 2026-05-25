# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 5)

### ✅ Phase 0-10: CORE PHASES COMPLETE

### Working Commands (full list):
```bash
# Banner
fm                    # Rich in terminal, raw when piped
fm banner --json      # JSON output
fm banner --raw       # Explicit raw output

# Env
fm env                # Project aliases (cargo run, npm run dev, etc.)

# File ops
fm mv <src> <dest> [--overwrite] [--rename]   # Move
fm cp <src> <dest> [--overwrite]             # Copy
fm rm <files> [-r]                            # Remove (safe)
fm trash <files>                              # Move to trash

# Clipboard
fm yank <files>              # Copy to clipboard
fm paste [--move-files]      # Paste from clipboard
fm clipboard [--clear]       # Show/clear clipboard

# Pins
fm pin <name>               # Bookmark directory
fm pins                     # List pins
fm jump <name>              # Print cd command
fm unpin <name>             # Remove pin
fm root                     # Jump to git root

# Utils
fm open <files>             # Open with default app
fm do [action]              # Act on stdin paths
fm stats [--json]           # Deep directory stats
fm diff <dir1> <dir2>        # Compare directories

# Sessions
fm save-session <name> [--desc]     # Save workspace
fm load-session <name>              # Show restore commands
fm sessions                         # List sessions
fm delete-session <name>            # Delete session

# Shell integration
fm install-hook            # Show hook script
fm completion <shell>      # Generate completions
fm config [--get] [--set]   # Config management
```

### ⏳ Phase 11: Session Management — DONE!
- `fm save-session` — Save cwd, branch, description
- `fm load-session` — Print restore commands
- `fm sessions` — List all saved sessions
- `fm delete-session` — Delete a session

### ⏳ Phase 12-18: Future phases
- Diff, completions, config, polish, tests, docs, packaging

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README