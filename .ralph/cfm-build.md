# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 6)

### ✅ Phase 0-12: ALL CORE + DIFF COMPLETE

### All Working Commands:
```bash
# Banner
fm                    # Rich in terminal, raw when piped
fm banner --json      # JSON output
fm banner --raw       # Explicit raw output

# Env
fm env                # Project aliases

# File ops
fm mv <src> <dest> [--overwrite] [--rename]
fm cp <src> <dest> [--overwrite]
fm rm <files> [-r]
fm trash <files>

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
fm open <files>             # Open with default app
fm do [action]              # Act on stdin paths
fm stats [--json]           # Deep directory stats
fm diff <dir1> <dir2>       # Compare directories

# Sessions
fm save-session <name> [--desc]
fm load-session <name>
fm sessions
fm delete-session <name>

# Shell integration
fm install-hook
fm completion <shell>
fm config [--get] [--set]
```

### ⏳ Remaining Phases
- Phase 13: Completions (clap_complete)
- Phase 14: Config (toml file support)
- Phase 15: Polish (clippy fixes, NO_COLOR, Unicode fallback)
- Phase 16: Tests
- Phase 17: Documentation (README update)
- Phase 18: Packaging (crates.io)

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README