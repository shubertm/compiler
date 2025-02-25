use serde::{Serialize, Deserialize};

/// JSON output structures
/// 
/// These structures are used to represent the compiled contract in a format
/// that can be serialized to JSON.

/// Parameter in a contract or function
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (pubkey, signature, bytes32, int, bool)
    #[serde(rename = "type")]
    pub param_type: String,
}

/// Script path for a function
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Operation {
    pub op: String,
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptPath {
    /// Function name
    pub function: String,
    /// Whether this is a server variant
    #[serde(rename = "serverVariant")]
    pub server_variant: bool,
    /// Bitcoin script operations
    pub operations: Vec<Operation>,
}

/// Main JSON output structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractJson {
    pub name: String,
    pub parameters: Vec<Parameter>,
    #[serde(rename = "serverKey")]
    pub server_key: String,
    #[serde(rename = "scriptPaths")]
    pub script_paths: Vec<ScriptPath>,
}

/// AST structures
/// 
/// These structures represent the parsed abstract syntax tree (AST)
/// of a TapLang contract.

/// Contract AST
#[derive(Debug)]
pub struct Contract {
    /// Contract name
    pub name: String,
    /// Contract parameters
    pub parameters: Vec<Parameter>,
    /// Contract functions
    pub functions: Vec<Function>,
}

/// Function AST
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Function arguments
    pub parameters: Vec<Parameter>,
    /// Function requirements
    pub requirements: Vec<Requirement>,
}

/// Requirement AST
#[derive(Debug, Clone)]
pub enum Requirement {
    /// Check signature requirement
    CheckSig { signature: String, pubkey: String },
    /// Check multisig requirement
    CheckMultisig { signatures: Vec<String>, pubkeys: Vec<String> },
    /// After requirement
    After { blocks: u64 },
    /// Hash equal requirement
    HashEqual { preimage: String, hash: String },
    /// Comparison requirement
    Comparison { left: Expression, op: String, right: Expression },
}

/// Expression AST
#[derive(Debug, Clone)]
pub enum Expression {
    /// Variable reference
    Variable(String),
    /// Literal value
    Literal(String),
    /// Property access (e.g., tx.time)
    Property(String),
    /// SHA256 hash function
    Sha256(String),
} 