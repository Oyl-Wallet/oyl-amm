# Technical Context: swap.oyl.io

## Technologies Used

### Programming Languages
- **Rust**: The primary programming language used for the entire project. Rust is chosen for its memory safety, performance, and strong type system.

### Frameworks and Libraries
- **Alkanes Framework**: A custom blockchain framework for Bitcoin or Bitcoin-related blockchains that enables complex smart contract functionality.
  - `alkanes-support`: Core support library for the Alkanes framework
  - `alkanes-runtime`: Runtime library for Alkanes contracts
  - `alkanes-std-factory-support`: Support library for factory patterns in Alkanes
  - `alkanes-runtime-pool`: Runtime library specifically for AMM pools
  - `alkanes-runtime-factory`: Runtime library specifically for AMM factories

- **Metashrew**: A framework that appears to be related to Bitcoin protocols
  - `metashrew-support`: Support library for the Metashrew framework
  - `metashrew-core`: Core library for the Metashrew framework

- **Protorune**: Another framework used in the project
  - `protorune-support`: Support library for the Protorune framework

### Bitcoin Libraries
- **bitcoin**: Rust library for Bitcoin functionality, used with the `rand` feature
- **ordinals**: Library for working with Bitcoin Ordinals

### Utility Libraries
- **anyhow**: Error handling library
- **hex**: Library for hexadecimal encoding/decoding
- **num**: Library for numeric types and operations
- **ruint**: Library for unsigned integer types
- **wasm-bindgen**: Library for WebAssembly bindings
- **wasm-bindgen-test**: Testing library for WebAssembly
- **hex_lit**: Library for hexadecimal literals
- **flate2**: Compression library

## Development Setup

### Project Structure
The project is organized as a Rust workspace with multiple crates:

```
swap.oyl.io/
├── src/
│   ├── lib.rs
│   └── tests/
│       ├── amm.rs
│       ├── fees.rs
│       ├── mod.rs
│       ├── precision_loss.rs
│       ├── swap_tests.rs
│       └── helper/
│           ├── add_liquidity.rs
│           ├── common.rs
│           ├── init_pools.rs
│           ├── mod.rs
│           ├── remove_liquidity.rs
│           └── swap.rs
├── alkanes/
│   ├── alkanes-runtime-factory/
│   ├── alkanes-runtime-pool/
│   ├── example-flashswap/
│   ├── factory/
│   ├── oyl-token/
│   └── pool/
├── Cargo.toml
└── build.rs
```

### Build System
- **Cargo**: Rust's package manager and build system
- **build.rs**: Custom build script for additional build steps

### Testing
- **wasm-bindgen-test**: Used for testing WebAssembly code
- Comprehensive test suite in the `src/tests` directory covering:
  - AMM functionality
  - Fee calculations
  - Precision loss scenarios
  - Swap operations
  - Helper functions for testing

## Technical Constraints

### Bitcoin Limitations
- Bitcoin's scripting language is intentionally limited, which is why the project uses the Alkanes framework to enable more complex functionality.
- The project must work within the constraints of the Bitcoin blockchain, such as block size limits and transaction fees.

### Precision and Rounding
- The project must handle precision loss and rounding carefully to avoid issues with token swaps and liquidity calculations.
- Tests specifically for precision loss scenarios are included in the test suite.

### Security Considerations
- The project must ensure that pools cannot be manipulated or exploited.
- Proper error handling and validation are essential to prevent security vulnerabilities.

## Dependencies

### External Dependencies
From the Cargo.toml file, the project depends on:

```toml
[workspace.dependencies]
alkanes-support = { git = "https://github.com/kungfuflex/alkanes-rs"}
alkanes-runtime = { git = "https://github.com/kungfuflex/alkanes-rs" }
alkanes-std-factory-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
metashrew-support = { git = "https://github.com/sandshrewmetaprotocols/metashrew" }
protorune-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
ordinals = { git = "https://github.com/kungfuflex/alkanes-rs" }
alkanes-runtime-pool = { path = "./alkanes/alkanes-runtime-pool" }
anyhow = "1.0.94"
bitcoin = { version = "0.32.4", features = ["rand"] }
hex = "0.4.3"
num = "0.4.3"
ruint = "1.13.1"
```

### Internal Dependencies
The project is organized as a workspace with multiple internal crates:

- **alkanes-runtime-factory**: Runtime support for the AMM factory
- **alkanes-runtime-pool**: Runtime support for AMM pools
- **example-flashswap**: Example implementation of a flash swap
- **factory**: Implementation of the AMM factory
- **oyl-token**: Implementation of the OYL token
- **pool**: Implementation of the AMM pool

## Tool Usage Patterns

### Development Tools
- **Cargo**: Used for building, testing, and managing dependencies
- **Git**: Used for version control
- **WebAssembly**: The project appears to target WebAssembly, as indicated by the use of wasm-bindgen

### Testing Patterns
- **Unit Tests**: Tests for individual components
- **Integration Tests**: Tests for interactions between components
- **Scenario Tests**: Tests for specific scenarios like precision loss
- **Helper Functions**: Reusable functions for common testing operations