# Lazy Flags Value Binding — folder-auto-banner

> **HISTORICAL**: This document describes the `:` value-binding syntax
> from 0.6.37. The entire lazy flag system was **removed in 0.7.0**
> in favor of the built-in alias system. See `LAZY_FLAGS_REMOVAL.md`
> for the current design. Retained for reference and git history.

**Date**: 2026-06-15
**Status**: Design phase — awaiting user decision.

## Problem

Lazy flag chains can contain **value-taking** flags. The current expansion
consumes subsequent arguments **in chain order**:

```text
f mLf 10 2 txt  →  f -m 10 -L 2 -f txt
                  (10 → -m, 2 → -L, txt → -f)
```

This works as long as the user supplies the **right number of values in the
right order**. But the chain alone does not tell you which flag a value binds
to, so ambiguity appears whenever the count does not match the number of
value-taking flags in the chain.

## Value-taking flags (current set)

| Char | Name  | Type   | Long form      |
|------|-------|--------|----------------|
| `m`  | max   | usize  | `--max <N>`    |
| `L`  | level | usize  | `--level <N>`  |
| `f`  | filter| string | `--filter <S>` |

(Plus the case-insensitive lowercase alias `l` → `L`.)

## Ambiguous cases

### Two value-taking flags, one value

```text
f mL 10      →  -m 10 -L          (10 binds to -m)
f Lm 10      →  -L 10 -m          (10 binds to -L)
```

**Problem**: if the user types `f mL 10`, did they mean `m=10, L=default` or
`m=default, L=10`? Reordering the chain resolves it, but that is fragile and
not obvious.

### Three value-taking flags, one value

```text
f mLf 10     →  -m 10 -L -f        (10 binds to -m)
f Lmf 10     →  -L 10 -m -f        (10 binds to -L)
f mLf 10     →  -m -L -f 10        (10 binds to -f, with the alternate rule we are considering)
```

**Three possible bindings, only one is chosen by the current rule (chain order).**

### Three value-taking flags, two values

```text
f mLf 10 2   →  -m 10 -L 2 -f      (10 → -m, 2 → -L)
f mLf 10 txt →  -m 10 -L -f txt    (10 → -m, txt → -L? or -f? — only 2 values, 3 flags)
```

When fewer values are supplied than value-taking flags, the trailing flags
get the default — the user may not realize which flag was skipped.

### Three value-taking flags, four values

```text
f mLf 10 2 txt rs
```

The fourth value (`rs`) has no value-taking flag to bind to. It becomes a
positional (path). This may or may not be what the user wanted.

## Current behavior summary

| Form                     | Expansion                            | Notes                           |
|--------------------------|--------------------------------------|---------------------------------|
| `f m 10`                 | `-m 10`                              | Unambiguous: one flag, one value |
| `f m 10 20`              | `-m 10 20`                           | `20` is treated as a path       |
| `f mL 10`                | `-m 10 -L`                           | **Ambiguous: which flag?**      |
| `f mL 10 2`              | `-m 10 -L 2`                         | Unambiguous if 2 values given  |
| `f mLf 10 2 txt`         | `-m 10 -L 2 -f txt`                  | Unambiguous if 3 values given  |
| `f mLf 10`               | `-m 10 -L -f`                        | **Ambiguous: which flag?**      |
| `f mLf 10 2`             | `-m 10 -L 2 -f`                      | `txt` is missing, ambiguous     |

## Why is this a real problem?

The user's question: **"what if we want to give the argument to the last one?"**

Example: a user types `f mLf 10` expecting to set the filter to `10` (perhaps
they made a typo and meant `txt`). With the current rule, `10` binds to `-m`
(max), not to `-f` (filter). The user gets a banner with max=10 and the
default filter — silently wrong.

A user who actually wants `f -f 10` has no way to express that with a chain
that includes `m` or `L` first. They have to fall back to explicit flags:
`f mLf -f 10` is invalid; `f mL -f 10` is also invalid because the routing
bypass for explicit flags (added in 0.6.34) treats any `-` as explicit-mode.

## Design alternatives

### A. Last-wins rule

**Rule**: All post-chain args bind to the LAST value-taking flag in the chain.
Flags before the last one use their default.

```text
f mLf 10         →  -m -L -f 10        (10 → -f)
f mLf 10 2 txt   →  -m -L -f 10 2 txt  (all three → -f — probably wrong)
```

**Trade-offs**
- ✅ Single value is unambiguous: "the last flag is what I am setting."
- ❌ Multiple values all go to the same flag (probably not intended).
- ❌ Setting 2 of 3 value-taking flags in one invocation becomes impossible.
- ❌ Backward incompatible: `f mL 10 2` (which currently means `m=10, L=2`) would change.

### B. Inline value syntax

**Rule**: A digit (or quoted string) immediately after a value-taking char
becomes its value, with no separator.

```text
f m10Lf2  →  -m 10 -L 2 -f
f m10     →  -m 10
```

**Trade-offs**
- ✅ Compact, no separator needed.
- ❌ Conflicts with single-char flags: `f m10` looks like `f m 1 0` (two flags).
- ❌ Conflicts with case-insensitive aliases: does `f m10L2` mean `m=10, l=2, f`? Or `m=10, L=2`?
- ❌ Hard to read for long values: `f m1000fhello` is not scannable.
- ❌ Requires explicit digit/quote boundary detection.

### C. Separator syntax

**Rule**: A `:` after a value-taking char marks it as "takes a value from the
next arg". Flags without `:` are boolean. This is closer to clap's `--flag=val`
syntax.

```text
f m:L:f: 10 2 txt  →  -m 10 -L 2 -f txt
f m:L 10           →  -m 10 -L
f mL 10            →  -m 10 -L          (m still consumes next arg, like today)
```

**Trade-offs**
- ✅ Explicit binding — no ambiguity.
- ✅ Backward compatible: bare `mL` still works as chain order.
- ❌ More characters: chain syntax is now `m:L:f:` instead of `mLf`.
- ❌ Users have to remember the colon rule.
- ❌ Inconsistent: sometimes you need `:` sometimes you don't.

### D. Per-flag binding with `=`

**Rule**: Allow `f m=10L=2f=txt` to bind explicitly. Bare `mLf` still works
(chain order, like today).

**Trade-offs**
- ✅ Most flexible.
- ❌ Most verbose.
- ❌ Two syntaxes to remember (bare and `=`).
- ❌ Mixing bare and `=` in one chain gets confusing.

### E. Keep current behavior, document workaround

**Rule**: No code change. Document that to bind a value to a specific flag,
the user should put that flag LAST in the chain (or use explicit `-m 10` form).

```text
# To set f=txt, m=default, L=default:
f Lmf txt  →  -L -m -f txt

# Or use explicit form (no lazy):
f -f txt
```

**Trade-offs**
- ✅ Zero code change, zero risk.
- ✅ The current rule IS deterministic — reordering the chain works.
- ❌ Users have to know the rule.
- ❌ "Put the flag you want to set last" is a non-obvious convention.

## Recommendation

**Option C (separator syntax)** is the best balance:
- Explicit binding when needed (`:m`).
- Backward compatible (bare `m` still consumes next arg).
- One consistent rule: `:` marks "I want a value".

**Option E** is the safest fallback if the user wants to avoid code changes.

## Chosen design: Option C (colon separator)

The user approved implementation. The chosen design is **Option C with a
refined syntax**: a `:` immediately after a value-taking flag in the chain
marks that flag as an **explicit value-binding target**. The next argument
in the command line binds to that flag.

### Rule

- `f mL` (no `:`) → chain order (current behavior, unchanged). If a value
  is available, it binds to `m`. Otherwise `m` uses its default.
- `f mL: 10` → the `:` after `L` marks `L` as the binding target. The next
  arg (`10`) binds to `L`. `m` uses its default.
- `f m:L: 10 2` → both `m` and `L` are binding targets. `10` → `m`, `2` → `L`.
  If a value-taking flag has no `:` and no available value, it uses default.
- `f m:L:f: 10 2 txt` → all three bind in chain order: `10`→`m`, `2`→`L`, `txt`→`f`.
- Mixing `:`, no `:`, and multiple values: values are consumed in chain
  order, but only flags marked with `:` are required to consume a value.

### Why this design

- ✅ **Solves the user's question**: `f mLf: 10` binds `10` to `f` (the last
  value-taking flag), so the user can choose which flag gets the value.
- ✅ **Backward compatible**: existing chains without `:` work exactly as
  before. The 0.6.34–0.6.36 test suite must still pass unchanged.
- ✅ **Explicit binding**: the `:` is a clear visual marker — no ambiguity.
- ✅ **Composable**: any subset of flags can be marked, the rest use defaults.
- ✅ **No new chars added to LAZY_FLAGS**: `:` is parsed in `expand_lazy_flags`
  as a separator, not a flag.
- ✅ **Preserves the no-fallback rule**: bare words are still lazy-flag
  chains, never paths.
- ✅ **Preserves case-insensitive aliases**: `l`→`L`, etc.
- ❌ Slightly more characters: `mLf:` vs `mLf`. Acceptable.

### Examples

| Form               | Expansion                  | Notes                              |
|--------------------|----------------------------|------------------------------------|
| `f m 10`           | `-m 10`                    | Unchanged (backward compat)        |
| `f mL 10`          | `-m 10 -L`                 | Unchanged (chain order)            |
| `f mL 10 2`        | `-m 10 -L 2`               | Unchanged (chain order)            |
| `f mL: 10`         | `-m -L 10`                 | `L` is the binding target          |
| `f :mL 10`         | `-m 10 -L`                 | `m` is the binding target (same as chain order) |
| `f m:L: 10 2`      | `-m 10 -L 2`               | Both marked, values bind in order  |
| `f mLf 10`         | `-m 10 -L -f`              | Unchanged (chain order)            |
| `f mLf: 10`        | `-m -L -f 10`              | `f` is the binding target — **answers the user's question** |
| `f m:L:f 10 2`     | `-m 10 -L 2 -f`            | `m` and `L` are targets; `f` uses chain order for the leftover value |
| `f mLf: 10 2 txt`  | `-m -L -f 10 2 txt`        | `f` is the target; `2` and `txt` are paths |

### Parsing rules

1. Iterate over the chars in the chain argument.
2. For each char that is a value-taking flag, check if the **next char** is `:`.
3. If yes, mark the flag as a binding target and skip the `:`.
4. If no, the flag follows chain-order behavior.
5. After parsing all flags, iterate through the chain flags in order.
   For each value-taking flag that is a binding target, consume the next
   arg as its value.
6. For value-taking flags that are NOT binding targets, follow chain-order:
   consume the next available arg if any, else use the default.

### What does NOT change

- `f t`, `f trc`, `f -t` — all unchanged.
- `f m 10`, `f mL 10 2` — unchanged (no `:` in chain).
- `expand_lazy_flags("trc")` — returns `Some(vec!['t', 'r', 'c'])` (unchanged).
- The function signature stays the same.
- The 0.6.34 routing bypass for explicit flags stays.
- The no-fallback rule stays.
- Case-insensitive aliases stay.

## Awaiting decision

The user approved going with **Option C (colon separator)**. Implementation
is in progress. See commit history for the change.
