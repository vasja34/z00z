#![cfg(not(target_arch = "wasm32"))]

#[tokio::test]
async fn test_examples_retired() {
    // Wallet examples were removed in favor of scenario_1 simulator flows.
    // Keep this target to preserve test grouping and CI shape.
}
