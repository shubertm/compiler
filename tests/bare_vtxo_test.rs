use taplang::compile;

#[test]
fn test_bare_vtxo_contract() {
    // Bare VTXO contract source code
    let vtxo_code = r#"contract BareVTXO(
  pubkey user,
  pubkey server,
  int timelock
) {
  // Cooperative spend path (user + server)
  function cooperative(signature userSig, signature serverSig) {
    require(checkMultisig([user, server], [userSig, serverSig]));
  }
  
  // Timeout path (user after timelock)
  function timeout(signature userSig) {
    require(checkSig(userSig, user));
    require(tx.time >= timelock);
  }
}"#;

    // Compile the contract
    let result = compile(vtxo_code);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let output = result.unwrap();
    
    // Verify contract name
    assert_eq!(output.name, "BareVTXO");
    
    // Verify parameters
    assert_eq!(output.parameters.len(), 3);
    assert_eq!(output.parameters[0].name, "user");
    assert_eq!(output.parameters[0].param_type, "pubkey");
    assert_eq!(output.parameters[1].name, "server");
    assert_eq!(output.parameters[1].param_type, "pubkey");
    assert_eq!(output.parameters[2].name, "timelock");
    assert_eq!(output.parameters[2].param_type, "int");
    
    // Verify script paths - we have 4 paths (2 functions x 2 variants)
    assert_eq!(output.script_paths.len(), 4);
    
    // Verify cooperative path
    let cooperative_path = output.script_paths.iter()
        .find(|p| p.function == "cooperative" && p.server_variant)
        .unwrap();
    
    // Check operations
    assert_eq!(cooperative_path.operations.len(), 7);
    assert_eq!(cooperative_path.operations[0].op, "OP_2");
    assert_eq!(cooperative_path.operations[1].op, "<user>");
    assert_eq!(cooperative_path.operations[2].op, "<server>");
    assert_eq!(cooperative_path.operations[3].op, "OP_2");
    
    // Verify timeout path
    let timeout_path = output.script_paths.iter()
        .find(|p| p.function == "timeout" && p.server_variant)
        .unwrap();
    
    // Check operations
    assert_eq!(timeout_path.operations.len(), 6);
    assert_eq!(timeout_path.operations[0].op, "<user>");
    assert_eq!(timeout_path.operations[1].op, "<userSig>");
    assert_eq!(timeout_path.operations[2].op, "OP_CHECKSIG");
} 