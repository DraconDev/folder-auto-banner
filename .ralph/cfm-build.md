# cfm Build Loop — Master TODO

Build the `cfm` (Contextual File Manager) Rust CLI tool.

## Progress (Iteration 5)

### ✅ Phase 0-11: ALL CORE PHASES COMPLETE

### Working Commands (full list):
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
fm do [action]              # Act on stdin paths (list/count/delete/open/cat/custom)
fm stats [--json]           # Deep directory stats

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

### Stats Output Features:
- Total size, files, directories, max depth
- File types breakdown with visual bars
- Largest files (top 10)
- Hidden files count, binary detection
- Code file percentage

### Do Command Actions:
- `fm do` (default: list) — List piped paths
- `fm do count` — Count lines in files
- `fm do delete` — Trash files
- `fm do open` — Open files
- `fm do cat` — Print file contents
- `fm do <cmd>` — Run custom command with `{}` for path

### ⏳ Phase 12-18: Polish phases
- Diff command (visual comparison)
- Completions (clap_complete)
- Config (toml)
- Polish (clippy fixes)
- Tests
- Documentation
- Packaging (crates.io)

## Done Criteria
1. Code compiles and passes `cargo test`
2. Feature works in both TTY and pipe modes
3. `--help` is complete for all subcommands
4. No clippy warnings
5. Documented in README