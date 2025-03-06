# TapLang

A compiler for TapLang - a simple, expressive language for creating Bitcoin Taproot scripts.

## Language Features

TapLang allows you to define Bitcoin Taproot contracts with:

- Strong typing (pubkey, signature, bytes32, int, bool)
- Multiple spending paths
- High-level expressions that compile to Bitcoin Script
- Block-based options for contract configuration
- Automatic server-variant path generation for settlement

## Installation

```bash
cargo install --path .
```

## Usage

```bash
tapc contract.tap
```

This will compile your TapLang contract to a JSON file that can be used with Bitcoin Taproot libraries.

## Contract Options

TapLang supports a block-based options approach for contract configuration:

```solidity
// Contract configuration options
options {
  // Server key parameter from contract parameters
  server = server;
  
  // Renewal timelock: 7 days (1008 blocks)
  renew = 1008;
  
  // Exit timelock: 24 hours (144 blocks)
  exit = 144;
}
```

Available options:

- `server`: Specifies which parameter contains the server public key
- `renew`: Specifies the renewal timelock in blocks
- `exit`: Specifies the exit timelock in blocks

## Example HTLC Contract

```solidity
// Contract configuration options
options {
  // Server key parameter from contract parameters
  server = server;
  
  // Exit timelock: 24 hours (144 blocks)
  exit = 144;
}

contract HTLC(
  pubkey sender,
  pubkey receiver,
  bytes32 hash,
  int refundTime,
  pubkey server
) {
  // Cooperative close path
  // This will automatically be compiled into two variants:
  // 1. serverVariant: true - requires multisig + server signature
  // 2. serverVariant: false - requires multisig + exit timelock
  function together(signature senderSig, signature receiverSig) {
    require(checkMultisig([sender, receiver], [senderSig, receiverSig]));
  }
  
  // Refund path
  // This will automatically be compiled into two variants:
  // 1. serverVariant: true - requires sender signature + refundTime + server signature
  // 2. serverVariant: false - requires sender signature + refundTime + exit timelock
  function refund(signature senderSig) {
    require(checkSig(senderSig, sender));
    require(tx.time >= refundTime);
  }
  
  // Claim path
  // This will automatically be compiled into two variants:
  // 1. serverVariant: true - requires receiver signature + valid preimage + server signature
  // 2. serverVariant: false - requires receiver signature + valid preimage + exit timelock
  function claim(signature receiverSig, bytes32 preimage) {
    require(checkSig(receiverSig, receiver));
    require(sha256(preimage) == hash);
  }
}
```

## Example Fuji Safe Contract

The Fuji Safe contract demonstrates advanced features of TapLang, including:

- Internal functions
- Error messages
- Transaction introspection
- Method chaining
- Complex expressions
- New data types for Taproot Assets

```solidity
// Contract configuration options
options {
  // Server key parameter from contract parameters
  server = treasuryPk;
  
  // Exit timelock: 24 hours (144 blocks)
  exit = 144;
}

// Fuji Safe Contract
contract FujiSafe(
  // The asset being borrowed
  asset borrowAsset,
  // The amount being borrowed
  int borrowAmount,
  // The borrower's public key
  pubkey borrowerPk,
  // The treasury's public key
  pubkey treasuryPk,
  // The expiration timeout in blocks
  int expirationTimeout,
  // The price level for liquidation
  int priceLevel,
  // The setup timestamp
  int setupTimestamp,
  // The oracle's public key
  pubkey oraclePk,
  // The asset pair identifier
  bytes32 assetPair
) {
  // Helper function to verify Fuji token burning
  function verifyFujiBurning() internal {
    // Verify output 0 is burning the Fuji token
    require(tx.output[0].asset == borrowAsset);
    require(tx.output[0].value == borrowAmount);
    require(tx.output[0].scriptPubKey.isOpReturn());
    require(tx.output[0].nonce == 0);
  }

  // Claim: Treasury can unlock all collateral after expiration when burning Fuji
  function claim(signature treasurySig) {
    // Check that expiration timeout has passed
    require(tx.time >= expirationTimeout, "Expiration timeout not reached");
    
    // Verify burning of Fuji token
    verifyFujiBurning();
    
    // Require treasury signature
    require(checkSig(treasurySig, treasuryPk), "Invalid treasury signature");
  }
  
  // Liquidation: Treasury can unlock all collateral with attestation price below the liquidation target
  function liquidate(int currentPrice, int timestamp, signature oracleSig, signature treasurySig) {
    // Check price is below liquidation threshold
    require(currentPrice < priceLevel, "Price not below liquidation threshold");
    
    // Verify timestamp is after setup
    require(timestamp >= setupTimestamp, "Timestamp before setup");
    
    // Verify oracle signature on price data
    bytes32 message = sha256(timestamp + currentPrice + assetPair);
    require(checkSigFromStack(oracleSig, oraclePk, message), "Invalid oracle signature");
    
    // Verify burning of Fuji token
    verifyFujiBurning();
    
    // Require treasury signature
    require(checkSig(treasurySig, treasuryPk), "Invalid treasury signature");
  }
  
  // Private Redemption: Only owner can unlock all collateral with key when burning Fuji
  function redeem(signature borrowerSig) {
    // Verify burning of Fuji token
    verifyFujiBurning();
    
    // Require borrower signature
    require(checkSig(borrowerSig, borrowerPk), "Invalid borrower signature");
  }
  
  // Treasury Renew: Treasury can unilaterally renew the expiration time
  function renew(signature treasurySig) {
    // Verify that output 0 has the same scriptPubKey, asset, and value as the current input
    require(tx.input.current.asset == tx.output[0].asset, "Asset mismatch");
    require(tx.input.current.value == tx.output[0].value, "Value mismatch");
    require(tx.input.current.scriptPubKey == tx.output[0].scriptPubKey, "ScriptPubKey mismatch");
    
    // Require treasury signature
    require(checkSig(treasurySig, treasuryPk), "Invalid treasury signature");
  }
}
```

### Advanced Features in Fuji Safe

1. **Internal Functions**: The `verifyFujiBurning()` function demonstrates code reuse.
2. **Error Messages**: Each `require` statement includes a descriptive error message.
3. **Transaction Introspection**: Access to transaction inputs and outputs.
4. **Method Chaining**: The `isOpReturn()` method on `scriptPubKey`.
5. **New Data Types**: `asset` type for Taproot assets.
6. **Complex Expressions**: Using intermediate variables for clarity.

## JSON Output Format

The compiler generates a JSON output with the following structure:

```json
{
  "contractName": "HTLC",
  "constructorInputs": [
    { "name": "sender", "type": "pubkey" },
    { "name": "receiver", "type": "pubkey" },
    { "name": "hash", "type": "bytes32" },
    { "name": "refundTime", "type": "int" },
    { "name": "server", "type": "pubkey" }
  ],
  "functions": [
    {
      "name": "claim",
      "functionInputs": [
        { "name": "receiverSig", "type": "signature" },
        { "name": "preimage", "type": "bytes32" }
      ],
      "serverVariant": true,
      "require": [
        { "type": "signature" },
        { "type": "hash" },
        { "type": "serverSignature" }
      ],
      "asm": [
        "<receiver>",
        "<receiverSig>",
        "OP_CHECKSIG",
        "<preimage>",
        "OP_SHA256",
        "<hash>",
        "OP_EQUAL",
        "<SERVER_KEY>",
        "<serverSig>",
        "OP_CHECKSIG"
      ]
    },
    {
      "name": "claim",
      "functionInputs": [
        { "name": "receiverSig", "type": "signature" },
        { "name": "preimage", "type": "bytes32" }
      ],
      "serverVariant": false,
      "require": [
        { "type": "signature" },
        { "type": "hash" },
        { "type": "older", "message": "Exit timelock of 144 blocks" }
      ],
      "asm": [
        "<receiver>",
        "<receiverSig>",
        "OP_CHECKSIG",
        "<preimage>",
        "OP_SHA256",
        "<hash>",
        "OP_EQUAL",
        "144",
        "OP_CHECKLOCKTIMEVERIFY",
        "OP_DROP"
      ]
    }
  ],
  "source": "...",
  "compiler": {
    "name": "taplang",
    "version": "0.1.0"
  },
  "updatedAt": "2025-03-05T23:28:28.331335+00:00"
}
```

Each function in the contract is compiled into two variants:

- `serverVariant: true`: Requires a server signature for settlement
- `serverVariant: false`: Requires an exit timelock for unilateral settlement

## Library Usage

You can also use TapLang as a library in your Rust projects:

```rust
use taplang::compile;

fn main() {
    let source_code = std::fs::read_to_string("contract.tap").unwrap();
    let result = compile(&source_code).unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
```

## License

MIT
