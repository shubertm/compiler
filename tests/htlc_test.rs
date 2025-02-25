use taplang::compile;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_htlc_contract() {
    // HTLC contract source code
    let htlc_code = r#"contract HTLC(
  pubkey sender,
  pubkey receiver,
  bytes32 hash,
  int timelock
) {
  function together(signature senderSig, signature receiverSig) {
    require(checkMultisig([sender, receiver], [senderSig, receiverSig]));
  }
  
  function refund(signature senderSig) {
    require(checkSig(senderSig, sender));
    require(tx.time >= timelock);
  }
  
  function claim(signature receiverSig, bytes32 preimage) {
    require(checkSig(receiverSig, receiver));
    require(sha256(preimage) == hash);
  }
}"#;

    // Compile the contract
    let result = compile(htlc_code);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let output = result.unwrap();
    
    // Verify contract name
    assert_eq!(output.name, "HTLC");
    
    // Verify parameters
    assert_eq!(output.parameters.len(), 4);
    assert_eq!(output.parameters[0].name, "sender");
    assert_eq!(output.parameters[0].param_type, "pubkey");
    assert_eq!(output.parameters[1].name, "receiver");
    assert_eq!(output.parameters[1].param_type, "pubkey");
    assert_eq!(output.parameters[2].name, "hash");
    assert_eq!(output.parameters[2].param_type, "bytes32");
    assert_eq!(output.parameters[3].name, "timelock");
    assert_eq!(output.parameters[3].param_type, "int");
    
    // Verify script paths - now we have 6 paths (3 functions x 2 variants)
    assert_eq!(output.script_paths.len(), 6);
    
    // Verify together path
    let together_path = output.script_paths.iter()
        .find(|p| p.function == "together" && p.server_variant)
        .unwrap();
    
    // Check operations
    assert_eq!(together_path.operations.len(), 7);
    assert_eq!(together_path.operations[0].op, "OP_2");
    assert_eq!(together_path.operations[1].op, "<sender>");
    assert_eq!(together_path.operations[2].op, "<receiver>");
    assert_eq!(together_path.operations[3].op, "OP_2");
    
    // Verify refund path
    let refund_path = output.script_paths.iter()
        .find(|p| p.function == "refund" && p.server_variant)
        .unwrap();
    
    // Check operations
    assert_eq!(refund_path.operations.len(), 6);
    assert_eq!(refund_path.operations[0].op, "<sender>");
    assert_eq!(refund_path.operations[1].op, "<senderSig>");
    assert_eq!(refund_path.operations[2].op, "OP_CHECKSIG");
    
    // Verify claim path
    let claim_path = output.script_paths.iter()
        .find(|p| p.function == "claim" && p.server_variant)
        .unwrap();
    
    // Check operations
    assert_eq!(claim_path.operations.len(), 7);
    assert_eq!(claim_path.operations[0].op, "<receiver>");
    assert_eq!(claim_path.operations[1].op, "<receiverSig>");
    assert_eq!(claim_path.operations[2].op, "OP_CHECKSIG");
}

#[test]
fn test_htlc_cli() {
    // Create a temporary directory for our test files
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("htlc.tap");
    let output_path = temp_dir.path().join("htlc.json");
    
    // HTLC contract source code
    let htlc_code = r#"contract HTLC(
  pubkey sender,
  pubkey receiver,
  bytes32 hash,
  int timelock
) {
  function together(signature senderSig, signature receiverSig) {
    require(checkMultisig([sender, receiver], [senderSig, receiverSig]));
  }
  
  function refund(signature senderSig) {
    require(checkSig(senderSig, sender));
    require(tx.time >= timelock);
  }
  
  function claim(signature receiverSig, bytes32 preimage) {
    require(checkSig(receiverSig, receiver));
    require(sha256(preimage) == hash);
  }
}"#;

    // Write the contract to a file
    fs::write(&input_path, htlc_code).unwrap();
    
    // Compile the contract using the library
    let result = compile(htlc_code);
    assert!(result.is_ok());
    
    let expected_output = result.unwrap();
    let expected_json = serde_json::to_string_pretty(&expected_output).unwrap();
    
    // Run the CLI command
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_tapc"))
        .arg(input_path.to_str().unwrap())
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .status()
        .expect("Failed to execute command");
    
    assert!(status.success());
    
    // Read the output file
    let actual_json = fs::read_to_string(&output_path).unwrap();
    
    // Compare the outputs
    assert_eq!(actual_json, expected_json);
} 