vadim@ASUS:~/Projects/z00z/crates/z00z_core$ cargo test --test claim_flow -- --nocapture
   Compiling z00z_core v0.1.0 (/home/vadim/Projects/z00z/crates/z00z_core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/genesis/claim_flow.rs (target/debug/deps/claim_flow-108f3a491b5b437a)

running 2 tests

🔐 CRYPTOGRAPHIC BALANCE VALIDATION TEST
========================================

🔧 Using cached genesis state...
📂 Loading genesis from binary fixture: "tests/genesis/fixtures/genesis_5000_coins.bin"
✅ Loaded 5000 coins from binary fixture (<1 sec)
✅ Genesis state loaded: 5,000 coins

📌 Claiming genesis coin at index 1234
   Context ID: [09, a0, b7, ea, 72, 4d, 95, 96]...
   Genesis commitment: HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)

🔑 Reconstructing wallet secrets from genesis seed...
✅ Genesis state built: 5,000 coins
📌 Claiming genesis coin at index 1234
   Context ID: [09, a0, b7, ea, 72, 4d, 95, 96]...
   Commitment found: HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)

🔑 Verifying context ID derivation...
   ✅ Context ID 0: [4a, 19, 31, 80, 35, 61, f4, 31]...
   ✅ Context ID 1: [53, ab, 70, c9, 20, 59, 87, cc]...
   ✅ Context ID 2: [0e, 82, d4, aa, 5b, 69, a0, 21]...
   ✅ Context ID 3: [f2, 67, 5e, ab, dd, b9, 24, d8]...
   ✅ Context ID 4: [da, 9d, 7e, 1a, 14, 84, f5, cb]...
   ✅ Context ID 5: [2d, 99, f8, c5, 5f, 58, d0, 1e]...
   ✅ Context ID 6: [f7, 6e, f6, 55, 34, bc, 0f, 4e]...
   ✅ Context ID 7: [d2, 82, 77, ad, 6f, 13, 94, 62]...
   ✅ Context ID 8: [da, 0f, 04, a1, 7c, bb, 5b, 08]...
   ✅ Context ID 9: [bb, 92, b7, 34, de, d7, 8a, a4]...

🔐 Verifying genesis coin properties...
   ✅ Lock height: None
   ✅ Is burn: false
   ✅ Nonce: non-zero

📊 Verifying state invariants...
   ✅ unspent output count: 5,000
   ✅ Spent count: 0
   ✅ Total supply: 1,000,000,000 base units

✅ Claim flow smoke test passed!
test test_claim_flow_smoke ... ok
   Genesis value: 200,000 base units
   Genesis blinding (reconstructed): [e6, e6, 14, 9d, 9b, bd, 64, 75]...

✅ CRYPTOGRAPHIC CHECK #1: Genesis commitment verification
   Verifying ownership: C_genesis == v·H + r·G
   Genesis commitment (from state): HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   Recomputed commitment:           HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   Match: true
   ✅ Wallet owns this coin (secrets match)

🔄 Creating transaction: 1 input → 2 outputs
   Input:  200,000 base units
   Output 1: 120,000 base units
   Output 2: 80,000 base units (change)

✅ CRYPTOGRAPHIC CHECK #2: Arithmetic balance
   Σ inputs:  200,000 base units
   Σ outputs: 200,000 base units
   Balance:   0 base units
   ✅ Arithmetic balance holds: Σ inputs == Σ outputs

✅ CRYPTOGRAPHIC CHECK #3: Pedersen commitment balance
   C_in:   HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   C_out1: HomomorphicCommitment(c062b86df5433d09fddbb154b8cfb32fa51d45524d2aa6bc546b91036a47331e)
   C_out2: HomomorphicCommitment(408a0730d3758cd70275b61007291264b779abf322d61a020820db495cae6232)
   Excess (C_in - C_out1 - C_out2): HomomorphicCommitment(0000000000000000000000000000000000000000000000000000000000000000)
   Expected (zero):                  HomomorphicCommitment(0000000000000000000000000000000000000000000000000000000000000000)
   Match: true
   ✅ Pedersen balance holds: Σ C_in - Σ C_out == 0

✅ CRYPTOGRAPHIC CHECK #4: Blinding factor balance
   r_in:     [e6, e6, 14, 9d, 9b, bd, 64, 75]...
   r_out1:   [96, d2, 2c, 25, 4a, e1, f4, 44]...
   r_out2:   [50, 14, e8, 77, 51, dc, 6f, 30]...
   r_excess: [00, 00, 00, 00, 00, 00, 00, 00]...
   r_zero:   [00, 00, 00, 00, 00, 00, 00, 00]...
   Match: true
   ✅ Blinding balance holds: Σ r_in - Σ r_out == 0

✅ CRYPTOGRAPHIC CHECK #5: Commitment construction formula
   Formula: C = v·H + r·G (Pedersen commitment)
   C_out1 (factory):  HomomorphicCommitment(c062b86df5433d09fddbb154b8cfb32fa51d45524d2aa6bc546b91036a47331e)
   C_out1 (manual):   HomomorphicCommitment(c062b86df5433d09fddbb154b8cfb32fa51d45524d2aa6bc546b91036a47331e)
   Match: true
   ✅ Commitment formula correct

✅ CRYPTOGRAPHIC CHECK #6: Homomorphic addition property
   Property: C(v1, r1) + C(v2, r2) = C(v1+v2, r1+r2)
   C_out1 + C_out2:        HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   C(v1+v2, r1+r2):        HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   Match: true
   ✅ Homomorphic property holds

✅ CRYPTOGRAPHIC CHECK #7: Input-output commitment equivalence
   Verify: C_in == C_out1 + C_out2 (when balanced)
   C_in:                   HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   C_out1 + C_out2:        HomomorphicCommitment(fcf5f104f91f3f0b2cd06814198e26addd4b06ad18965e58af2285b1604cc126)
   Match: true
   ✅ Input equals sum of outputs (in commitment space)

📊 CRYPTOGRAPHIC VALIDATION SUMMARY
====================================
✅ Genesis commitment verified
✅ Arithmetic balance: 200,000 == 200,000
✅ Pedersen balance: C_in - C_out1 - C_out2 == 0
✅ Blinding balance: r_in - r_out1 - r_out2 == 0
✅ Commitment formula: C = v·H + r·G
✅ Homomorphic property: C(v1,r1) + C(v2,r2) = C(v1+v2,r1+r2)
✅ Input-output equivalence: C_in == Σ C_out

🎉 All cryptographic checks passed!
   Transaction is cryptographically sound and balanced.

test test_claim_with_cryptographic_balance_validation ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

vadim@ASUS:~/Projects/z00z/crates/z00z_core$ 