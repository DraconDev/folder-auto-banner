#!/usr/bin/env bash
# test_lazy_flags.sh — Standalone test harness for the lazy flag system.
#
# Runs key lazy flag examples and verifies they produce the same exit code
# as the explicit form. Byte-identical comparison is done by the Rust tests.
#
# Usage: ./scripts/test_lazy_flags.sh
# Exit code: 0 on all pass, 1 on any failure.

set -uo pipefail

# Find the f binary
F_BIN="${F_BIN:-./target/release/f}"
if [[ ! -x "$F_BIN" ]]; then
    F_BIN="$(which f 2>/dev/null || true)"
fi
if [[ ! -x "$F_BIN" ]]; then
    echo "ERROR: Cannot find 'f' binary. Set F_BIN or build with 'cargo build --release'."
    exit 1
fi

PASS=0
FAIL=0
FAILED_TESTS=()

# Run f with args, capture exit code and a small sample of output
run_f() {
    local args=("$@")
    local exit_code
    "$F_BIN" "${args[@]}" </dev/null >/dev/null 2>&1
    exit_code=$?
    echo $exit_code
}

# Verify lazy form and explicit form have the same exit code
test_pair() {
    local lazy_args=("$@")
    local explicit_args=()

    # Build explicit form
    local chain="${lazy_args[0]}"
    # Apply lowercase alias mapping
    case "$chain" in
        s) chain="S" ;;
        g) chain="G" ;;
        d) chain="D" ;;
        l) chain="L" ;;
        u) chain="U" ;;
    esac

    # Expand chain, handling value-taking flags (m, f, L) by consuming next arg
    local expanded=""
    local value_idx=1  # index into lazy_args for values
    for ((i=0; i<${#chain}; i++)); do
        local c="${chain:$i:1}"
        if [[ -z "$expanded" ]]; then
            expanded="-$c"
        else
            expanded="$expanded -$c"
        fi
        # If value-taking, consume the next arg as the value
        if [[ "$c" == "m" || "$c" == "f" || "$c" == "L" ]]; then
            if [[ $value_idx -lt ${#lazy_args[@]} ]]; then
                expanded="$expanded ${lazy_args[$value_idx]}"
                ((value_idx++))
            fi
        fi
    done

    eval "set -- $expanded"
    for a in "$@"; do
        explicit_args+=("$a")
    done
    # Add any remaining args (paths, etc.)
    for ((i=value_idx; i<${#lazy_args[@]}; i++)); do
        explicit_args+=("${lazy_args[$i]}")
    done

    local test_name="f ${lazy_args[*]} ≡ f ${explicit_args[*]}"
    local lazy_exit explicit_exit
    lazy_exit=$(run_f "${lazy_args[@]}")
    explicit_exit=$(run_f "${explicit_args[@]}")

    if [[ "$lazy_exit" == "$explicit_exit" ]]; then
        ((PASS++))
    else
        ((FAIL++))
        FAILED_TESTS+=("$test_name (lazy=$lazy_exit, explicit=$explicit_exit)")
    fi
}

echo "=== Standalone Lazy Flag Test Harness ==="
echo "Binary: $F_BIN"
echo ""

# Test: single flags (14 boolean, 3 value-taking)
for c in a c D e G o r R S t U v x X; do
    test_pair "$c"
done

# Test: lowercase aliases (4 boolean, 1 value-taking)
for c in s g d u; do
    test_pair "$c"
done

# Test: value-taking singles (should error — value required)
for c in m f L; do
    test_pair "$c"  # both should fail with "value required"
done

# Test: chained flags
test_pair "tr"
test_pair "trc"
test_pair "tS"
test_pair "GS"
test_pair "Rc"
test_pair "rS"
test_pair "ta"
test_pair "aR"
test_pair "oR"
test_pair "Dt"

# Test: value-taking chains
test_pair "m" "10"
test_pair "L" "2"
test_pair "f" "txt"
test_pair "mL" "10" "2"
test_pair "tSm" "10"
test_pair "mLf" "10" "2" "txt"

echo ""
echo "=== Results ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"

if [[ $FAIL -gt 0 ]]; then
    echo ""
    echo "Failed tests:"
    for t in "${FAILED_TESTS[@]}"; do
        echo "  - $t"
    done
    exit 1
fi

echo ""
echo "All tests passed! (byte-identical verification done by Rust tests)"
exit 0
