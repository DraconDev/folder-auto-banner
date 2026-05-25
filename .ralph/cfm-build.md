# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 6)

### ✅ Phase 0-13: ALMOST COMPLETE!

### All Working Commands:
```bash
# Banner
fm                    # Rich in terminal, raw when piped
fm banner --json      # JSON output

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
fm pin <name>         # Bookmark directory
fm pins              # List pins
fm jump <name>       # Print cd command
fm unpin <name>      # Remove pin
fm root              # Jump to git root

# Utils
fm open <files>       # Open with default app
fm do [action]        # Act on stdin paths
fm stats [--json]     # Deep directory stats
fm diff <dir1> <dir2> # Compare directories

# Sessions
fm save-session <name> [--desc]
fm load-session <name>
fm sessions
fm delete-session <name>

# Shell integration
fm install-hook       # Show hook script
fm completion <shell> # Generate completions (bash/zsh/fish/ps/elvish)
fm config [--get] [--set]
```

### ⏳ Remaining Phases
- Phase 14: Config (toml file support) — stub exists
- Phase 15: Polish (clippy fixes, NO_COLOR, Unicode fallback) — ~30 warnings
- Phase 16: Tests — none yet
- Phase 17: Documentation — basic README exists
- Phase 18: Packaging (crates.io)

## Done Criteria
1. Code compiles and passes `cargo test` — ❌ no tests
2. Feature works in both TTY and pipe modes — ✅
3. `--help` is complete for all subcommands — ✅
4. No clippy warnings — ❌ ~30 warnings
5. Documented in README — ✅ basic docs