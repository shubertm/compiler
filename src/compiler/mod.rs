use crate::models::{Requirement, Expression, ContractJson, ScriptPath, Operation};
use crate::parser;

/// Compiles a TapLang contract AST into a JSON-serializable structure.
/// 
/// This function takes a parsed Contract AST and transforms it into a ContractJson
/// structure that can be serialized to JSON. The output includes:
/// 
/// - Contract name
/// - Parameters
/// - Server key placeholder
/// - Script paths for each function
/// 
/// Each script path includes a serverVariant flag. When using the script:
/// - If serverVariant is true, use the script as-is
/// - If serverVariant is false, libraries should add an exit delay timelock
///   (default 48 hours) for additional security
/// 
/// # Arguments
/// 
/// * `source_code` - The source code of the contract
/// 
/// # Returns
/// 
/// A Result containing a ContractJson structure that can be serialized to JSON or an error message
pub fn compile(source_code: &str) -> Result<ContractJson, String> {
    let contract = match parser::parse(source_code) {
        Ok(contract) => contract,
        Err(err) => return Err(format!("Parse error: {}", err)),
    };

    // Create output structure
    let mut output = ContractJson {
        name: contract.name.clone(),
        parameters: contract.parameters.clone(),
        server_key: "SERVER_KEY".to_string(),
        script_paths: Vec::new(),
    };

    // Create script paths for each function
    let mut script_paths = Vec::new();
    for function in &contract.functions {
        // Generate server variant
        let mut operations = Vec::new();
        
        // Process requirements
        for req in &function.requirements {
            match req {
                &Requirement::CheckSig { ref pubkey, ref signature } => {
                    // Push pubkey
                    operations.push(Operation {
                        op: format!("<{}>", pubkey),
                        data: None,
                    });
                    
                    // Push signature
                    operations.push(Operation {
                        op: format!("<{}>", signature),
                        data: None,
                    });
                    
                    // Check signature
                    operations.push(Operation {
                        op: "OP_CHECKSIG".to_string(),
                        data: None,
                    });
                }
                &Requirement::CheckMultisig { ref pubkeys, ref signatures } => {
                    // Push number of signatures
                    operations.push(Operation {
                        op: format!("OP_{}", signatures.len()),
                        data: None,
                    });
                    
                    // Push each pubkey
                    for pubkey in pubkeys {
                        operations.push(Operation {
                            op: format!("<{}>", pubkey),
                            data: None,
                        });
                    }
                    
                    // Push number of pubkeys
                    operations.push(Operation {
                        op: format!("OP_{}", pubkeys.len()),
                        data: None,
                    });
                    
                    // Push each signature
                    for sig in signatures {
                        operations.push(Operation {
                            op: format!("<{}>", sig),
                            data: None,
                        });
                    }
                    
                    // Check multisig
                    operations.push(Operation {
                        op: "OP_CHECKMULTISIG".to_string(),
                        data: None,
                    });
                }
                &Requirement::After { blocks } => {
                    // Push blocks
                    operations.push(Operation {
                        op: blocks.to_string(),
                        data: None,
                    });
                    
                    // Check timelock
                    operations.push(Operation {
                        op: "OP_CHECKLOCKTIMEVERIFY".to_string(),
                        data: None,
                    });
                    
                    // Drop the value
                    operations.push(Operation {
                        op: "OP_DROP".to_string(),
                        data: None,
                    });
                }
                &Requirement::HashEqual { ref preimage, ref hash } => {
                    // Push preimage
                    operations.push(Operation {
                        op: format!("<{}>", preimage),
                        data: None,
                    });
                    
                    // Hash it
                    operations.push(Operation {
                        op: "OP_SHA256".to_string(),
                        data: None,
                    });
                    
                    // Push hash
                    operations.push(Operation {
                        op: format!("<{}>", hash),
                        data: None,
                    });
                    
                    // Check equality
                    operations.push(Operation {
                        op: "OP_EQUAL".to_string(),
                        data: None,
                    });
                }
                &Requirement::Comparison { ref left, op: _, ref right } => {
                    match left {
                        Expression::Sha256(preimage) => {
                            // Push preimage
                            operations.push(Operation {
                                op: format!("<{}>", preimage),
                                data: None,
                            });
                            
                            // Hash it
                            operations.push(Operation {
                                op: "OP_SHA256".to_string(),
                                data: None,
                            });
                            
                            // Push hash value from right
                            if let Expression::Variable(hash) = right {
                                operations.push(Operation {
                                    op: format!("<{}>", hash),
                                    data: None,
                                });
                            }
                            
                            // Check equality
                            operations.push(Operation {
                                op: "OP_EQUAL".to_string(),
                                data: None,
                            });
                        }
                        Expression::Property(prop) if prop == "tx.time" => {
                            // Handle timelock comparison
                            if let Expression::Variable(timelock) = right {
                                // Push timelock value
                                if let Ok(blocks) = timelock.parse::<u64>() {
                                    operations.push(Operation {
                                        op: blocks.to_string(),
                                        data: None,
                                    });
                                } else {
                                    operations.push(Operation {
                                        op: format!("<{}>", timelock),
                                        data: None,
                                    });
                                }
                                
                                // Check timelock
                                operations.push(Operation {
                                    op: "OP_CHECKLOCKTIMEVERIFY".to_string(),
                                    data: None,
                                });
                                
                                // Drop the value
                                operations.push(Operation {
                                    op: "OP_DROP".to_string(),
                                    data: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Add script path for this function
        script_paths.push(ScriptPath {
            function: function.name.clone(),
            server_variant: true,
            operations: operations.clone(),
        });
        
        // Also add a non-server variant with server_variant flag set to false
        // Libraries will need to add the exit delay timelock when using this variant
        let non_server_operations = operations.clone();
        script_paths.push(ScriptPath {
            function: function.name.clone(),
            server_variant: false,
            operations: non_server_operations,
        });
    }

    output.script_paths = script_paths;
    Ok(output)
} 