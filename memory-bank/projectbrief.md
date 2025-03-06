# Swap.oyl.io Project Brief

## Project Overview

Swap.oyl.io is a decentralized exchange (DEX) platform implementing an Automated Market Maker (AMM) system for cryptocurrency trading. The project provides smart contracts for generic AMM pools and a specific implementation called Oylswap. It is built on a custom blockchain runtime environment called Alkanes, which appears to be a WASM-based smart contract platform.

The core functionality includes:
- Creating and managing liquidity pools for token pairs
- Swapping tokens through these pools
- Adding and removing liquidity
- Routing trades through multiple pools for optimal execution

The project follows a design pattern similar to Uniswap, with factory contracts that create and manage pool contracts, and a router that handles multi-hop swaps and liquidity operations.

## Technical Context

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

### System Architecture

The system follows a modular architecture with several key components:

1. **AMM Pool Implementation**:
   - Defined in `alkanes-runtime-pool`
   - Implements the core AMM functionality including:
     - Liquidity provision and removal
     - Token swapping with fee calculation
     - Reserve management
     - Price calculation

2. **Factory Contract**:
   - Defined in `alkanes-runtime-factory`
   - Creates and manages pool contracts
   - Maintains a registry of created pools
   - Ensures uniqueness of pools for token pairs

3. **Router Contract**:
   - Handles user interactions
   - Routes trades through multiple pools if necessary
   - Provides optimized paths for token swaps
   - Manages liquidity operations

4. **Specialized Implementations**:
   - OYL-specific pool and factory implementations
   - Support for different blockchain networks via feature flags

### Design Patterns

1. **Factory Pattern**: Used for creating and managing pool instances
2. **Delegate Pattern**: Used for runtime behavior customization
3. **Constant Product Formula**: x * y = k formula for AMM pricing
4. **Trait-based Polymorphism**: Used for implementing different pool behaviors

## Source Code Modules

### Core Modules

1. **alkanes-runtime-pool**
   - Implements the core AMM pool functionality
   - Defines the `AMMPoolBase` trait with methods for:
     - Pool initialization
     - Liquidity provision (mint)
     - Liquidity removal (burn)
     - Token swapping
     - Reserve management
   - Uses constant product formula (x * y = k) for price calculation
   - Implements a 0.4% swap fee (DEFAULT_FEE_AMOUNT_PER_1000 = 4)

2. **alkanes-runtime-factory**
   - Implements the factory contract for creating and managing pools
   - Defines the `AMMFactoryBase` trait with methods for:
     - Factory initialization
     - Pool creation
     - Pool lookup
   - Maintains a registry of created pools
   - Ensures uniqueness of pools for token pairs

3. **router**
   - Implements the router contract for user interactions
   - Handles multi-hop swaps
   - Manages liquidity operations through the factory and pools
   - Provides optimized paths for token swaps

### Implementation Modules

1. **pool**
   - Concrete implementation of the AMM pool
   - Uses the `declare_alkane!` macro to create the WASM contract

2. **factory**
   - Concrete implementation of the AMM factory
   - Uses the `declare_alkane!` macro to create the WASM contract

3. **oyl-pool** and **oyl-factory**
   - OYL-specific implementations of the pool and factory contracts
   - May contain customizations for the OYL token ecosystem

### Test Modules

1. **src/tests**
   - Contains comprehensive tests for the AMM functionality
   - Tests include:
     - Pool initialization
     - Liquidity provision and removal
     - Token swapping
     - Edge cases and error conditions
   - Uses the `wasm-bindgen-test` framework for WASM testing

## Additional Context

### Build System
- Uses Cargo for dependency management and building
- Custom `build.rs` script for WASM compilation and optimization
- Feature flags for different network configurations:
  - `test`: For testing environment
  - `testnet`: For testnet deployment
  - `mainnet`: For mainnet deployment
  - Various coin-specific features (dogecoin, luckycoin, bellscoin, fractal)

### Testing Strategy
- Comprehensive unit tests for core functionality
- Integration tests for end-to-end workflows
- WASM-specific testing using `wasm-bindgen-test`
- Test fixtures for common test scenarios

### Deployment
- Compiled to WASM for on-chain deployment
- Supports multiple network configurations via feature flags
- Uses compression for optimizing WASM binary size

### Integration Notes
- Integrates with Bitcoin-related functionality
- Uses custom balance sheet implementation for token accounting
- Implements the constant product formula (x * y = k) for AMM pricing
- Uses a 0.4% swap fee by default 