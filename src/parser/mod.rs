use pest::Parser;
use pest_derive::Parser;
use pest::iterators::{Pair, Pairs};
use crate::models::{Contract, Function, Parameter, Requirement, Expression};

// Grammar definition for pest parser
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct TapLangParser;

pub fn parse(source_code: &str) -> Result<Contract, Box<dyn std::error::Error>> {
    let pairs = TapLangParser::parse(Rule::main, source_code)?;
    let ast = build_ast(pairs);
    Ok(ast)
}

// Parse pest output into AST
fn build_ast(pairs: Pairs<Rule>) -> Contract {
    let mut contract = Contract {
        name: String::new(),
        parameters: Vec::new(),
        functions: Vec::new(),
    };
    
    for pair in pairs {
        match pair.as_rule() {
            // Main rule contains the contract
            Rule::main => {
                // Find the contract inside main
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() == Rule::contract {
                        parse_contract(&mut contract, inner_pair);
                    }
                }
            }
            // Direct contract rule (for backward compatibility)
            Rule::contract => {
                parse_contract(&mut contract, pair);
            }
            // Skip other rules
            _ => {}
        }
    }
    
    contract
}

// Helper function to parse contract details
fn parse_contract(contract: &mut Contract, pair: Pair<Rule>) {
    let mut inner_pairs = pair.into_inner();
    
    // Contract name
    contract.name = inner_pairs.next().unwrap().as_str().to_string();
    
    // Parameters
    let param_list = inner_pairs.next().unwrap();
    for param_pair in param_list.into_inner() {
        if param_pair.as_rule() == Rule::parameter {
            let mut param_inner = param_pair.into_inner();
            let param_type = param_inner.next().unwrap().as_str().to_string();
            let param_name = param_inner.next().unwrap().as_str().to_string();
            
            contract.parameters.push(Parameter {
                name: param_name,
                param_type: param_type,
            });
        }
    }
    
    // Functions
    for func_pair in inner_pairs {
        if func_pair.as_rule() == Rule::function {
            let func = parse_function(func_pair);
            contract.functions.push(func);
        }
    }
}

// Parse function from pest output
fn parse_function(pair: Pair<Rule>) -> Function {
    let mut func = Function {
        name: String::new(),
        parameters: Vec::new(),
        requirements: Vec::new(),
    };
    
    let mut inner_pairs = pair.into_inner();
    
    // Function name
    func.name = inner_pairs.next().unwrap().as_str().to_string();
    
    // Parameters
    let param_list = inner_pairs.next().unwrap();
    for param_pair in param_list.into_inner() {
        if param_pair.as_rule() == Rule::parameter {
            let mut param_inner = param_pair.into_inner();
            let param_type = param_inner.next().unwrap().as_str().to_string();
            let param_name = param_inner.next().unwrap().as_str().to_string();
            
            func.parameters.push(Parameter {
                name: param_name,
                param_type: param_type,
            });
        }
    }
    
    // Requirements
    for req_pair in inner_pairs {
        if req_pair.as_rule() == Rule::require_stmt {
            let expr_pair = req_pair.into_inner().next().unwrap();
            let requirement = parse_expression(expr_pair);
            func.requirements.push(requirement);
        }
    }
    
    func
}

// Parse expression from pest output
fn parse_expression(pair: Pair<Rule>) -> Requirement {
    match pair.as_rule() {
        Rule::check_sig => {
            let mut inner = pair.into_inner();
            let signature = inner.next().unwrap().as_str().to_string();
            let pubkey = inner.next().unwrap().as_str().to_string();
            
            Requirement::CheckSig { signature, pubkey }
        }
        Rule::check_multisig => {
            let mut inner = pair.into_inner();
            let pubkeys_array = inner.next().unwrap();
            let sigs_array = inner.next().unwrap();
            
            let mut pubkeys = Vec::new();
            for pubkey in pubkeys_array.into_inner() {
                pubkeys.push(pubkey.as_str().to_string());
            }
            
            let mut signatures = Vec::new();
            for sig in sigs_array.into_inner() {
                signatures.push(sig.as_str().to_string());
            }
            
            Requirement::CheckMultisig { signatures, pubkeys }
        }
        Rule::time_comparison => {
            // Handle tx.time >= timelock
            let mut inner = pair.into_inner();
            let blocks = inner.next().unwrap().as_str().parse::<u64>().unwrap_or(0);
            
            Requirement::After { blocks }
        }
        Rule::hash_comparison => {
            // Handle sha256(preimage) == hash
            let mut inner = pair.into_inner();
            let sha256_func = inner.next().unwrap();
            let hash = inner.next().unwrap().as_str().to_string();
            
            // Extract preimage from sha256_func
            let preimage = sha256_func.into_inner().next().unwrap().as_str().to_string();
            
            Requirement::HashEqual { preimage, hash }
        }
        _ => {
            // Default to a comparison for other cases
            Requirement::Comparison {
                left: Expression::Variable("unknown".to_string()),
                op: "==".to_string(),
                right: Expression::Variable("unknown".to_string()),
            }
        },
    }
} 