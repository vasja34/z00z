const WASM_CLIENT_SRC: &str = include_str!("../src/wasm_client.rs");

#[test]
fn test_wasm_src_redaction() {
    assert!(!WASM_CLIENT_SRC.contains("RPC call: {} with params: {}"));
    assert!(!WASM_CLIENT_SRC.contains("RPC response: {}"));
    assert!(!WASM_CLIENT_SRC.contains("Connecting to worker: {}"));
    assert!(!WASM_CLIENT_SRC.contains("Connected to worker: {}"));
    assert!(WASM_CLIENT_SRC.contains("req_log(method, &params)"));
    assert!(WASM_CLIENT_SRC.contains("resp_log(&response)"));
}
