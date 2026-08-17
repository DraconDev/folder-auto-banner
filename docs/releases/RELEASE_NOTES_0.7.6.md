# Release Notes — f 0.7.6

**Date**: 2026-06-15

## Doc cleanup: removed historical lazy flag design docs

The repository had 5 `LAZY_FLAGS_*.md` files describing the lazy
flag system that was removed in 0.7.0. Four of them were pure
historical record of a system that only existed in dev for a
few hours and never had external users. The fifth (the design
doc) has been renamed to reflect the current alias system.

## Changes

### Removed (1132 lines)

- `LAZY_FLAGS_AUDIT.md` (360 lines) — audit of the removed system
- `LAZY_FLAGS_MESSINESS.md` (269 lines) — messiness analysis
- `LAZY_FLAGS_TESTING.md` (224 lines) — test plan
- `LAZY_FLAGS_VALUE_BINDING.md` (271 lines) — design for the
  `:` value-binding syntax that was never shipped externally

### Renamed

- `LAZY_FLAGS_REMOVAL.md` → `ALIASES.md` (the design doc for
  the current alias system, kept and updated header)

### Updated

- `CHANGELOG.md` — references to `LAZY_FLAGS_REMOVAL.md` updated
  to `ALIASES.md`

## No code changes

This is a documentation-only release. No behavior changes.

## Files changed

- `LAZY_FLAGS_AUDIT.md` — **deleted**
- `LAZY_FLAGS_MESSINESS.md` — **deleted**
- `LAZY_FLAGS_TESTING.md` — **deleted**
- `LAZY_FLAGS_VALUE_BINDING.md` — **deleted**
- `LAZY_FLAGS_REMOVAL.md` → `ALIASES.md` (renamed, header updated)
- `CHANGELOG.md` — entry for 0.7.6
- `Cargo.toml` — version bump 0.7.5 → 0.7.6
- `f.1` — version bump 0.7.5 → 0.7.6

## Installation

```bash
cargo install folder-auto-banner --version 0.7.6 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
