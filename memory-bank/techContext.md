# Swap.oyl.io Technical Context

## Technologies Used

### Programming Languages and Technologies

- **Rust**: The primary programming language used throughout the codebase
- **WebAssembly (WASM)**: Used for smart contract compilation and execution
- **Cargo**: Rust package manager and build system

### Frameworks and Libraries

- **alkanes-runtime**: Custom runtime environment for smart contract execution
- **alkanes-support**: Support library for the Alkanes runtime
- **metashrew-support**: Support library for data indexing and storage
- **protorune-support**: Support library for protocol-specific functionality
- **anyhow**: Error handling library
- **bitcoin**: Bitcoin-related functionality, including transaction handling
- **ruint**: Unsigned integer library for large number operations
- **num**: Numerical computation library
- **wasm-bindgen**: WebAssembly binding library for testing

### Development Tools

- **Rust Toolchain**: Required for building the project
  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **wasm32-unknown-unknown Target**: Required for compiling to WebAssembly
  ```
  rustup target add wasm32-unknown-unknown
  ```

- **wasm-bindgen-cli**: Required for testing WebAssembly code
  ```
  cargo install -f wasm-bindgen-cli --version 0.2.99
  ```

## Development Setup

### Prerequisites

1. **Rust Toolchain**: Install the Rust programming language and Cargo package manager
2. **WebAssembly Target**: Add the WebAssembly target to the Rust toolchain
3. **Git**: For version control and accessing the repository

### Building the Project

The project can be built with Cargo, specifying the desired network:

```sh
cargo build --release --features all,<network>
```

Where `<network>` is one of:
- `mainnet`: Bitcoin mainnet
- `testnet`: Bitcoin testnet
- `regtest`: Bitcoin regtest
- `dogecoin`: Dogecoin network
- `luckycoin`: Luckycoin network
- `bellscoin`: Bellscoin network
- `fractal`: Fractal network

The build produces:
- WASM contracts in `target/wasm32-unknown-unknown/release/`

### Testing

The project includes several testing approaches:

1. **Unit Tests**: Testing individual components
   ```
   cargo test -p <crate-name>
   ```

2. **Integration Tests**: Testing interactions between components
   ```
   cargo test
   ```

3. **WASM Tests**: Testing the compiled WebAssembly
   ```
   cargo test --all
   ```

## Technical Constraints

### Bitcoin Compatibility

Swap.oyl.io must operate within the constraints of the Bitcoin protocol:
- Limited transaction size
- Limited script capabilities
- No native smart contract support
- Immutable transaction history

### WebAssembly Limitations

Smart contracts must operate within WebAssembly constraints:
- Limited memory model
- No direct system access
- Deterministic execution
- Limited floating-point precision

### ALKANES Metaprotocol Constraints

The ALKANES metaprotocol imposes its own constraints:
- Specific addressing system for contracts
- Limited fuel/gas for computation
- Protocol-specific message formats
- Storage limitations

### Performance Considerations

Performance is a critical consideration for the system:
- Efficient use of storage
- Minimizing computational complexity
- Optimizing for gas/fuel usage
- Handling large numbers efficiently

## Dependencies

### Core Dependencies

- **alkanes-runtime**: Provides the execution environment for ALKANES smart contracts
  ```toml
  alkanes-runtime = { git = "https://github.com/kungfuflex/alkanes-rs" }
  ```

- **alkanes-support**: Core utilities for the ALKANES protocol
  ```toml
  alkanes-support = { git = "https:/github.com/kungfuflex/alkanes-rs" }
  ```

- **metashrew-support**: Utilities for the METASHREW indexer stack
  ```toml
  metashrew-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
  ```

- **protorune-support**: Utilities for the protorunes protocol
  ```toml
  protorune-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
  ```

- **ordinals**: Support for Bitcoin ordinals
  ```toml
  ordinals = { git = "https://github.com/kungfuflex/alkanes-rs" }
  ```

### Standard Library Dependencies

- **alkanes-runtime-pool**: AMM pool implementation
  ```toml
  alkanes-runtime-pool = { path = "./alkanes/alkanes-runtime-pool" }
  ```

- **alkanes-runtime-factory**: AMM factory implementation
  ```toml
  alkanes-runtime-factory = { path = "./alkanes/alkanes-runtime-factory" }
  ```

### External Dependencies

- **anyhow**: Error handling library
  ```toml
  anyhow = "1.0.94"
  ```

- **bitcoin**: Bitcoin data structures and utilities
  ```toml
  bitcoin = { version = "0.32.4", features = ["rand"] }
  ```

- **hex**: Hexadecimal encoding/decoding
  ```toml
  hex = "0.4.3"
  ```

- **num**: Numerical computation utilities
  ```toml
  num = "0.4.3"
  ```

- **ruint**: Unsigned integer library for large numbers
  ```toml
  ruint = "1.13.1"
  ```

### Development Dependencies

- **alkanes**: ALKANES metaprotocol implementation
  ```toml
  alkanes = { git = "https://github.com/kungfuflex/alkanes-rs", features = ["test-utils"] }
  ```

- **metashrew**: METASHREW indexer stack
  ```toml
  metashrew = { git = "https://github.com/kungfuflex/alkanes-rs", features = ["test-utils"] }
  ```

- **protorune**: Protorunes protocol implementation
  ```toml
  protorune = { git = "https://github.com/kungfuflex/alkanes-rs", features = ["test-utils"] }
  ```

- **wasm-bindgen**: WebAssembly bindings for Rust
  ```toml
  wasm-bindgen = "0.2.100"
  ```

- **wasm-bindgen-test**: Testing framework for WebAssembly
  ```toml
  wasm-bindgen-test = "0.3.49"
  ```

- **hex_lit**: Hexadecimal literals
  ```toml
  hex_lit = "0.1.1"
  ```

## Build System

The project uses Cargo as its build system, with custom build scripts for:

1. **WASM Compilation**: Compiling Rust code to WebAssembly
   ```rust
   // build.rs
   fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Build logic...
       Ok(())
   }
   ```

2. **WASM Optimization**: Optimizing the WebAssembly binary size
   ```rust
   // Using flate2 for compression
   use flate2::write::GzEncoder;
   use flate2::Compression;
   ```

3. **Feature Flags**: Conditional compilation based on network and features
   ```toml
   [features]
   test = []
   testnet = []
   dogecoin = []
   luckycoin = []
   bellscoin = []
   fractal = []
   mainnet = []
   ```

## Integration with ALKANES Ecosystem

Swap.oyl.io integrates with the broader ALKANES ecosystem:

### ALKANES Metaprotocol

The ALKANES metaprotocol provides the foundation for smart contract execution on Bitcoin:
- **Addressing System**: Contracts are addressed by their AlkaneId (block and tx fields)
- **Message System**: Inter-contract communication through messages
- **Storage System**: Persistent storage for contract state
- **Execution Environment**: WebAssembly-based execution environment

### METASHREW Indexer

The METASHREW indexer provides blockchain data indexing and querying:
- **Block Processing**: Processing Bitcoin blocks and transactions
- **State Management**: Maintaining the state of the ALKANES ecosystem
- **Query Interface**: Providing RPC methods for external access
- **View Functions**: Implementing query functions for contract state

### Protorunes Protocol

The protorunes protocol provides the token standard:
- **Token Creation**: Creating new tokens
- **Token Transfer**: Transferring tokens between addresses
- **Balance Management**: Managing token balances
- **Rune Standard**: Implementing the rune token standard

## Deployment Considerations

### Resource Requirements

- **CPU**: Moderate to high, especially during initial synchronization
- **Memory**: Moderate, depending on the size of the state
- **Storage**: High, as the blockchain and state grow
- **Network**: Moderate, for blockchain synchronization

### Security Considerations

- **Fuel Metering**: Prevents DoS attacks through resource exhaustion
- **Sandboxed Execution**: Isolates contract execution from the host system
- **Input Validation**: Ensures only valid transactions are processed
- **Error Handling**: Gracefully handles invalid inputs and execution failures

### Monitoring and Maintenance

- **Logs**: The system produces logs for debugging and monitoring
- **State Backup**: Regular backups of the state are recommended
- **Version Compatibility**: Updates should maintain compatibility with existing contracts
- **Network Synchronization**: The indexer must stay synchronized with the blockchain

## View Functions and Trace Data

View functions provide a way to query the state of the system without modifying it. The ALKANES system uses a specific mechanism for returning data from view functions through trace data.

### View Function Implementation

View functions in the ALKANES system are implemented as opcodes in the contract's `execute` method:

```rust
impl AlkaneResponder for AMMFactory {
    fn execute(&self) -> Result<CallResponse> {
        if let Some(delegate) = &self.delegate {
            let context = self.context()?;
            let mut inputs = context.inputs.clone();
            match shift_or_err(&mut inputs)? {
                // Existing opcodes...
                3 => delegate.get_all_pools(), // View function
                // Other opcodes...
                _ => Err(anyhow!("unrecognized opcode")),
            }
        } else {
            Err(anyhow!("No delegate set"))
        }
    }
}
```

### View Function Response Format

View functions return their results in the `data` field of the `CallResponse` struct:

```rust
fn get_all_pools(&self) -> Result<CallResponse> {
    let length = self.all_pools_length()?;
    let mut response = CallResponse::default();
    let mut all_pools_data = Vec::new();
    
    // Add the total count as the first element
    all_pools_data.extend_from_slice(&length.to_le_bytes());
    
    // Add each pool ID
    for i in 0..length {
        match self.all_pools(i) {
            Ok(pool_id) => {
                all_pools_data.extend_from_slice(&pool_id.block.to_le_bytes());
                all_pools_data.extend_from_slice(&pool_id.tx.to_le_bytes());
            }
            Err(_) => {
                // Skip any errors and continue
                continue;
            }
        }
    }
    
    response.data = all_pools_data;
    Ok(response)
}
```

### Trace Data Mechanism

When a view function is called, the result is captured in the transaction's trace data:

1. **Execution Process**:
   - The contract's `execute` method is called with the appropriate opcode
   - The corresponding function (e.g., `get_all_pools`) is executed
   - The function returns a `CallResponse` with the result in the `data` field
   - The ALKANES runtime captures this response in the transaction's trace data

2. **Trace Data Structure**:
   - The trace data is stored in `vout 3` of the transaction
   - It includes a header section with execution context information
   - The actual return data is located at an offset within the trace data
   - The format of the return data depends on the specific function

3. **Accessing Trace Data**:
   - Use the `view::trace()` function with the transaction outpoint
   - The trace data is returned as a byte array
   - Parse the byte array based on knowledge of the data format

### Parsing Trace Data

To parse trace data from a view function:

```rust
// Get the trace data from vout 3
let outpoint = OutPoint {
    txid: transaction.compute_txid(),
    vout: 3,
};
let trace_data = view::trace(&outpoint)?;

// Parse the data (example for get_all_pools)
// Note: The actual offset may vary between functions
const DATA_OFFSET: usize = 87; // This offset is specific to get_all_pools

// Parse the count
let count_bytes: [u8; 16] = trace_data[DATA_OFFSET..DATA_OFFSET+16]
    .try_into()
    .map_err(|_| anyhow::anyhow!("Failed to read count"))?;
let count = u128::from_le_bytes(count_bytes);

// Parse each item
let mut items = Vec::new();
for i in 0..count {
    let item_offset = DATA_OFFSET + 16 + (i as usize * 32); // 16 bytes for count, 32 bytes per item
    
    // Parse the item (example for AlkaneId)
    let block_bytes: [u8; 16] = trace_data[item_offset..item_offset+16]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read block ID"))?;
    let block = u128::from_le_bytes(block_bytes);
    
    let tx_bytes: [u8; 16] = trace_data[item_offset+16..item_offset+32]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read tx ID"))?;
    let tx = u128::from_le_bytes(tx_bytes);
    
    items.push(AlkaneId::new(block, tx));
}
```

### Client-Side Integration

On the client side, view functions are called through the ALKANES RPC interface:

```typescript
// Call the view function
const response = await alkanesRpc.call({
    method: "metashrew_view",
    params: ["get_all_pools", "0x...", "latest"]
});

// Parse the response
function parseGetAllPoolsResponse(response: Uint8Array): AlkaneId[] {
    const pools: AlkaneId[] = [];
    
    // Find the data section (may require knowledge of the trace structure)
    const dataOffset = 87; // This offset is specific to get_all_pools
    
    // Read the count
    const countBuffer = response.slice(dataOffset, dataOffset + 16);
    const count = readUint128FromBuffer(countBuffer);
    
    // Read each pool ID
    let offset = dataOffset + 16;
    for (let i = 0; i < count; i++) {
        const blockBuffer = response.slice(offset, offset + 16);
        offset += 16;
        const txBuffer = response.slice(offset, offset + 16);
        offset += 16;
        
        const block = readUint128FromBuffer(blockBuffer);
        const tx = readUint128FromBuffer(txBuffer);
        
        pools.push({ block, tx });
    }
    
    return pools;
}
```

## Testing Patterns and Best Practices

Based on our debugging experience with the "get all pools" functionality, we've identified several important testing patterns and common issues that are critical for developing and maintaining the Swap.oyl.io system.

### Testing Best Practices

1. **Clear State Between Tests**:
   - Always use `clear()` at the beginning of each test to reset the state
   - This prevents interference between tests and "already holds a binary" errors
   ```rust
   #[wasm_bindgen_test]
   fn test_function() -> Result<()> {
       clear();
       // Test implementation...
   }
   ```

2. **Proper Factory Initialization**:
   - Initialize the factory only once per test
   - Use `init_block_with_amm_pool()` for basic setup without creating pools
   - Use `test_amm_pool_init_fixture()` when you need pools created
   ```rust
   // When you only need the factory initialized:
   let (block, deployment_ids) = init_block_with_amm_pool()?;
   
   // When you need pools created:
   let (block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
   ```

3. **Trace Data Inspection**:
   - Use `view::trace()` to inspect the result of contract calls
   - Convert trace data to structured formats when possible
   ```rust
   let trace_result: Trace = view::trace(&outpoint)?.try_into()?;
   println!("Trace result: {:?}", trace_result);
   ```

4. **Error Handling in Tests**:
   - Use descriptive assertions to make test failures clear
   - Handle expected errors gracefully
   ```rust
   assert!(!trace.is_empty(), "No trace data returned from get_all_pools call");
   ```

5. **Test Function Organization**:
   - Group related tests in the same file
   - Use helper functions for common operations
   - Follow a consistent pattern: setup, action, verification

### Common Testing Issues

We've encountered several common issues during testing:

1. **Storage Conflicts**:
   - **Symptom**: "used CREATERESERVED cellpack but X already holds a binary"
   - **Cause**: Multiple tests using the same storage keys without clearing
   - **Solution**: Call `clear()` at the beginning of each test

2. **Factory Initialization Errors**:
   - **Symptom**: "ALKANES: revert: must send two runes to initialize a pool"
   - **Cause**: Attempting to create a pool without proper initialization or with incorrect parameters
   - **Solution**: Ensure factory is initialized and exactly two tokens are provided

3. **Missing Trace Data**:
   - **Symptom**: "No trace data returned from get_all_pools call"
   - **Cause**: The contract call didn't produce any trace data, possibly due to an error
   - **Solution**: Check contract implementation and ensure proper error handling

4. **Import Errors**:
   - **Symptom**: "could not find `X` in `Y`"
   - **Cause**: Missing or incorrect imports
   - **Solution**: Check import paths and ensure all dependencies are correctly imported

5. **Test Interference**:
   - **Symptom**: Tests pass individually but fail when run together
   - **Cause**: Shared state between tests
   - **Solution**: Ensure each test properly cleans up after itself

## Documentation and Resources

### ALKANES Specification

The ALKANES specification is hosted in the project wiki:
- [https://github.com/kungfuflex/alkanes-rs/wiki](https://github.com/kungfuflex/alkanes-rs/wiki)

### Protorunes Documentation

Documentation for protorunes is available at:
- [https://github.com/kungfuflex/protorune/wiki](https://github.com/kungfuflex/protorune/wiki)

### METASHREW Documentation

The METASHREW indexer stack is documented at:
- [https://github.com/sandshrewmetaprotocols/metashrew](https://github.com/sandshrewmetaprotocols/metashrew)
