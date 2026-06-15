# Release Notes — f 0.7.5

**Date**: 2026-06-15

## README: removed migration table from 0.6.x

The README had a "Migration from 0.6.x" section that mapped lazy
flag chains (e.g. `f trc`) to 0.7+ aliases (e.g. `f new -r -c`).
The lazy flag system only existed in dev for a few hours before
the alias system replaced it; it never had external users.

## Changes

### Removed

- The "Migration from 0.6.x" subsection header
- The 6-row migration table
- The intro paragraph

### Kept

- The "Built-in Aliases" section (the actual current API)
- The shell alias example (still useful for personal shortcuts)

## No code changes

This is a documentation-only release. No behavior changes.

## Files changed

- `README.md` — migration table removed (-14 lines).
- `CHANGELOG.md` — entry for 0.7.5.
- `Cargo.toml` — version bump 0.7.4 → 0.7.5.
- `f.1` — version bump 0.7.4 → 0.7.5.

## Installation

```bash
cargo install folder-auto-banner --version 0.7.5 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
