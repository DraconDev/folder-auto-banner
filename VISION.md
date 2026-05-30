# cfm — Vision & Scope

## What We Are

**A directory listing with instant context.**

When you `cd` into a directory, you see:
- File listing (like `ls`/`exa`/`lsd`)
- Git status
- Build status
- TODO count
- Project type
- Ports in use
- Docker status

**All instantly, no extra commands needed.**

## What We're Not

- Not a `z`/`zoxide` replacement (they're already perfect)
- Not a `fd`/`find` replacement (fd is already perfect)
- Not a `cp`/`mv`/`rm` replacement (people have their own tools)
- Not a "do everything" tool

## Our Edge

**Instant context.** When you `cd` into a project, you immediately see:
- Is the build passing?
- How many TODOs?
- What ports are in use?
- Is Docker running?

**No other tool does this.** That's our value.

## Core Features

### Directory Listing
- File permissions, owner, group, size, date, name
- Icons for file types
- Git status per file (modified, added, deleted, untracked)
- Sorting by name, size, date, type
- Hidden file toggle
- Tree view

### Context (The Differentiator)
- Git: branch, status, ahead/behind, stash count
- Build: pass/fail, errors, warnings
- TODOs: count by type (TODO, FIXME, HACK)
- Metrics: total LOC, files by type
- Ports: listening ports for this project
- Docker: running containers

### Output Modes
- Rich (default): colored, formatted
- Raw: plain text for piping
- JSON: for scripting

## Commands

| Command | Description |
|---------|-------------|
| `f` | Directory listing + context |
| `f <dir>` | Listing for specific dir |
| `f --sort name\|size\|date\|type` | Sort order |
| `f --reverse` | Reverse sort |
| `f --hidden` | Show dotfiles |
| `f --tree [depth]` | Tree view |
| `f --json` | JSON output |
| `f --raw` | Plain text output |
| `f --compact` | Less info |
| `f --filter <pattern>` | Filter by name |

## What We Removed

Commands that were removed because they duplicate existing tools:
- `cp`, `mv`, `rm`, `trash` — people have their own
- `yank`, `paste`, `clipboard` — niche
- `pin`, `unpin`, `pins` — redundant with frecency
- `save-session`, `load-session`, `sessions`, `delete-session` — over-engineering
- `diff` — people use `diff` or `meld`
- `do`, `peek` — niche
- `stats` — covered by banner
- `config` — keep simple, use env vars

## Design Principles

1. **Fast** — daemon caches pre-computed data
2. **Instant context** — no extra commands needed
3. **Not a replacement** — we're complementary, not competitive
4. **Simple** — few commands, clear purpose
5. **Useful** — show what people actually need

## Success Metric

**People use `f` instead of `ls`/`exa`/`lsd` because they get more context instantly.**
