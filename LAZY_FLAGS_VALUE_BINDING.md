# Lazy Flags Value Binding — folder-auto-banner

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

## Awaiting decision

The user must choose:
- **A, B, C, D** — implement the new design.
- **E** — keep current behavior, just document the workaround.

No code changes will be made until the user explicitly approves a design.
