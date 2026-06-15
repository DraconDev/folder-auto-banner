# Release Notes — f 0.7.4

**Date**: 2026-06-15

## README and INSTALL.md updated for 0.7.x routing

The user-facing docs were out of date for the 0.7.x routing system.
This release updates them to reflect the current behavior.

## Changes

### README.md

- **Replaced** the "Lazy Flags" section (0.6.x, removed) with
  a new "Built-in Aliases" section listing all 18 aliases and
  their flag expansions.
- **Added** a new "Routing rules (0.7+)" section documenting
  the three accepted input types (numbers, aliases, flags) and
  the `-b` banner switch.
- **Updated** the "Numbered Navigation" section to note that a
  number is the only non-alias bare word that produces a result.
- **Updated** the "Usage" examples to use `f -b ./src` instead
  of the obsolete `f <dir>` (paths are now dropped in non-banner
  mode).
- **Added** a migration table from 0.6.x lazy flag chains to
  0.7+ aliases (`f t` → `f new`, `f trc` → `f new -r -c`, etc.).
- **Added** a new "`-b` Banner Mode (0.7.3+)" section with
  examples for path-specific banners.
- **Removed** outdated test marker comments.

### INSTALL.md

- **Fixed** `f ~/Downloads` → `f -b ~/Downloads` (paths are now
  dropped in non-banner mode; use `-b` for path-specific
  banners).

## No code changes

This is a documentation-only release. No behavior changes.

## Files changed

- `README.md` — full rewrite of Usage, Aliases, and Navigation
  sections; migration table added.
- `INSTALL.md` — `f ~/Downloads` → `f -b ~/Downloads`.
- `CHANGELOG.md` — entry for 0.7.4.
- `Cargo.toml` — version bump 0.7.3 → 0.7.4.
- `f.1` — version bump 0.7.3 → 0.7.4.

## Installation

```bash
cargo install folder-auto-banner --version 0.7.4 --locked --force
```

Or update an existing install:

```bash
cargo install folder-auto-banner --force
```
