# folder-auto-banner 0.6.23

## Summary
Documentation cleanup for the current install flow, daemon management, shell-wrapper navigation, and background size-refresh behavior.

## What changed
- README now describes `f` as a contextual directory dashboard and clarifies that the first cold view of very large directories may briefly show `4.0k` placeholders while background refresh completes.
- INSTALL now uses the current `./install.sh` and `f install` flow instead of outdated manual hook snippets.
- `f(1)` man page now targets `f 0.6.23` and documents current commands, daemon subcommands, options, environment variables, and examples.

## Validation
- Documentation diff inspected.
- No runtime code behavior changes introduced.
- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo doc --no-deps`
- `cargo build --release --locked`
- `cargo publish --dry-run --locked`
