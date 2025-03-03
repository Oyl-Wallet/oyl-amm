# Technical Context: swap.oyl.io

## Technology Stack

### Core Technologies
- **Rust**: The primary programming language used for development
- **Bitcoin**: The underlying blockchain platform
- **Alkanes Metaprotocol**: The protocol layer that enables smart contract functionality on Bitcoin
- **Protorunes**: Token standard created by burning Runes and sending them to smart contract IDs
- **Metashrew**: Backend infrastructure that powers contract interactions and indexing

### Dependencies
The project relies on several key dependencies:

```
[dependencies]
alkanes-support = { git = "https:/github.com/kungfuflex/alkanes-rs" }
alkanes-runtime = { git = "https://github.com/kungfuflex/alkanes-rs" }
metashrew-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
protorune-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
ordinals = { git = "https://github.com/kungfuflex/alkanes-rs" }
alkanes-runtime-pool = { path = "./alkanes/alkanes-runtime-pool" }
anyhow = "1.0.94"
bitcoin = { version = "0.32.4", features = ["rand"] }
hex = "0.4.3"
num = "0.4.3"
ruint = "1.13.1"
```

## Project Structure

The project is organized as a Rust workspace with multiple crates:

```
swap.oyl.io/
├── Cargo.toml                  # Main workspace configuration
├── build.rs                    # Build script
├── src/                        # Main crate source
│   ├── lib.rs                  # Library entry point
│   └── tests/                  # Test suite
│       ├── amm.rs              # AMM functionality tests
│       ├── mod.rs              # Test module definition
│       └── helper/             # Test helpers
│           ├── add_liquidity.rs
│           ├── common.rs
│           ├── init_pools.rs
│           ├── mod.rs
│           ├── remove_liquidity.rs
│           └── swap.rs
└── alkanes/                    # Workspace members
    ├── alkanes-runtime-factory/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # Factory runtime implementation
    ├── alkanes-runtime-pool/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # Pool runtime implementation
    ├── factory/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # Factory implementation
    ├── oyl-factory/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # OYL-specific factory implementation
    ├── oyl-pool/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # OYL-specific pool implementation
    ├── pool/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs          # Pool implementation
    └── router/
        ├── Cargo.toml
        └── src/
            └── lib.rs          # Router implementation
```

## Key Technical Concepts

### Alkanes Metaprotocol

The Alkanes metaprotocol is a layer built on top of Bitcoin that enables smart contract functionality. It works by:

1. Defining a set of rules for interpreting Bitcoin transactions
2. Using a custom indexer to track the state of contracts
3. Providing a runtime environment for executing contract logic

In the context of swap.oyl.io, Alkanes provides the foundation for implementing the AMM logic.

### Protorunes

Protorunes are tokens created by burning Runes (a Bitcoin token standard) and sending them to smart contract IDs. They serve as the token standard for swap.oyl.io, allowing users to swap between different tokens.

### AMM Implementation

The AMM implementation follows the constant product formula (x * y = k) used by Uniswap V2. Key aspects include:

1. **Liquidity Pools**: Each pool contains reserves of two tokens and maintains the constant product invariant.
2. **Swap Pricing**: Prices are determined by the ratio of reserves, with a 0.4% fee applied to each swap.
3. **LP Tokens**: Liquidity providers receive LP tokens representing their share of the pool.
4. **Router**: The router handles multi-hop swaps and directs operations to the appropriate pools.

### Zero-Sum Logic

The system enforces zero-sum logic, ensuring that tokens cannot be created or destroyed within the system. This is crucial for maintaining the integrity of the token supply.

## Development Environment

### Build System
The project uses Cargo, Rust's package manager and build system. The workspace configuration in `Cargo.toml` defines the project structure and dependencies.

### Testing
The project includes a comprehensive test suite in the `src/tests` directory. Tests cover various aspects of the AMM functionality, including:

1. Pool initialization
2. Adding and removing liquidity
3. Swapping tokens
4. Multi-hop routing
5. Edge cases and error handling

Tests use the `wasm_bindgen_test` framework, allowing them to be run in both native and WebAssembly environments.

### Deployment
The project is designed to be deployed on the Bitcoin network using the Alkanes metaprotocol. The deployment process involves:

1. Compiling the Rust code to WebAssembly
2. Deploying the WebAssembly bytecode to the Alkanes runtime
3. Initializing the factory and router contracts

## Technical Constraints

### Bitcoin Limitations
Bitcoin's limited scripting capabilities impose constraints on the implementation. The Alkanes metaprotocol works around these limitations, but certain design decisions are influenced by the underlying Bitcoin architecture.

### Gas Efficiency
While Bitcoin doesn't have a concept of gas like Ethereum, the Alkanes metaprotocol introduces a fuel system to prevent infinite loops and ensure efficient execution. The implementation must be optimized for fuel efficiency.

### Integer Arithmetic
The implementation uses fixed-point arithmetic with 128-bit and 256-bit integers to handle token amounts and calculations. This introduces precision considerations, especially for very large or very small token amounts.

### Concurrency
The Bitcoin blockchain processes transactions in blocks, which introduces concurrency challenges. The implementation must handle potential race conditions and ensure consistent state transitions.

## Integration Points

### Alkanes Runtime
The project integrates with the Alkanes runtime through the `alkanes-runtime` and `alkanes-support` crates. These provide the foundation for implementing the smart contract logic.

### Metashrew Backend
The Metashrew backend powers the contract interactions and indexing. It provides utilities for handling Bitcoin transactions and maintaining the state of the contracts.

### Bitcoin Network
The ultimate integration point is the Bitcoin network itself. The implementation must adhere to Bitcoin's transaction format and consensus rules.

## Performance Considerations

### Swap Efficiency
Swaps should be executed efficiently to minimize computational overhead and maximize throughput. The implementation uses optimized algorithms for calculating swap outputs.

### Multi-hop Routing
Multi-hop swaps involve multiple pool interactions, which can be computationally expensive. The router implementation is optimized to minimize the number of hops and ensure efficient execution.

### Reserve Management
Managing token reserves is a critical aspect of the AMM. The implementation uses efficient data structures and algorithms to track and update reserves.