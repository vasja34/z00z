#!/bin/bash
# Fast tests only - skips slow genesis tests with 64-bit Bulletproofs+ generation
#
# Fast tests (<10 seconds):
#   - Unit tests (lib)
#   - multi_asset_genesis (7 tests, ~0.01s)
#   - determinism (3 non-ignored tests, ~0.25s)
#   - claim_flow (1 test, ~0.1s)
#   - range_proofs (2 non-ignored tests, ~1.4s)
#   - golden_snapshot (1 non-ignored test, ~0.1s)
#   - commitment_sum (3 tests, ~10s)
#   - crypto_security (12 tests, ~22s)
#
# Skipped slow tests (>30 seconds each):
#   - stress_test::test_genesis_medium_scale_1k_coins (~150s)
#   - stress_test::test_genesis_large_scale_10k_coins (~600s)
#   - stress_test::test_genesis_xlarge_scale_50k_coins (~2000s)
#   - production_scale tests (full 50k coin generation)

set -e  # Exit on error

echo "🚀 Running fast test suite..."
echo ""

echo "📚 Unit tests (lib)..."
cargo test --lib --quiet

echo ""
echo "✅ Asset genesis tests (multi-type)..."
cargo test --test multi_asset_genesis --quiet

echo ""
echo "🔁 Determinism tests (fixture-based)..."
cargo test --test determinism --quiet

echo ""
echo "💰 Claim flow tests..."
cargo test --test claim_flow --quiet

echo ""
echo "🔐 Range proof tests (non-ignored)..."
cargo test --test range_proofs --quiet

echo ""
echo "📸 Golden snapshot tests (non-ignored)..."
cargo test --test golden_snapshot --quiet

echo ""
echo "🧮 Commitment sum validation..."
cargo test --test commitment_sum --quiet

echo ""
echo "🔒 Crypto security tests..."
cargo test --test crypto_security --quiet

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All fast tests passed!"
echo ""
echo "💡 To run slow stress tests (1k+ coins):"
echo "   cargo test --test stress_test -- --ignored"
echo ""
echo "💡 To run all tests:"
echo "   cargo test --all-targets"

