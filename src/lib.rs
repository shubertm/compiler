pub mod models;
pub mod parser;
pub mod compiler;

pub use models::{Contract, Function, Parameter, Requirement, Expression, ContractJson, ScriptPath};

/// Compile TapLang source code to a JSON-serializable structure
///
/// This function takes TapLang source code as input, parses it into an AST,
/// and then compiles it into a ContractJson structure that can be serialized to JSON.
///
/// The output includes:
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
/// * `source_code` - The TapLang source code as a string
///
/// # Returns
///
/// A Result containing either the ContractJson structure or an error
///
/// # Example
///
/// ```
/// use taplang::compile;
///
/// let source_code = r#"contract Example(pubkey owner) {
///     function spend(signature ownerSig) {
///         require(checkSig(ownerSig, owner));
///     }
/// }"#;
///
/// let result = compile(source_code);
/// assert!(result.is_ok());
///
/// // Serialize to JSON
/// let json = serde_json::to_string_pretty(&result.unwrap()).unwrap();
/// println!("{}", json);
/// ```
pub fn compile(source_code: &str) -> Result<ContractJson, Box<dyn std::error::Error>> {
    match compiler::compile(source_code) {
        Ok(output) => Ok(output),
        Err(err) => Err(err.into()),
    }
} 