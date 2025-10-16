# OYL AMM

This repository contains the smart contracts for `swap.oyl.io`, a decentralized exchange (DEX) that implements an Automated Market Maker (AMM) model, similar to Uniswap, but designed for Bitcoin-based blockchains using the Alkanes framework.

## High-Level Overview

`swap.oyl.io` enables users to trade tokens and provide liquidity in a decentralized and permissionless manner. The core of the protocol is a set of smart contracts that allow for the creation of liquidity pools, swapping of tokens, and earning fees for liquidity providers.

The system is built on the following key principles:
- **Decentralization**: No central party controls the exchange or user funds.
- **Automated Market Making**: Token prices are determined by the ratio of assets in a liquidity pool, using the constant product formula (`x * y = k`).
- **Permissionless**: Anyone can create a market for a new token pair, provide liquidity, or swap tokens.

## Directory Structure

The project is organized as a Rust workspace with several crates, each responsible for a specific part of the system.

```
oyl-protocol/
├── alkanes/
│   ├── alkanes-runtime-factory/ # Core logic for the AMM factory
│   ├── alkanes-runtime-pool/    # Core logic for AMM pools
│   ├── example-flashswap/       # Example implementation of a flash swap
│   ├── factory/                 # Interface for the factory contract
│   ├── oyl-token/               # Implementation of the OYL token
│   ├── oylswap-library/         # Shared library code for oylswap
│   └── pool/                    # Interface for the AMM pool contracts
├── memory-bank/                 # Project documentation and context
├── prod_wasms/                  # Compiled WASM binaries for production
├── src/
│   ├── lib.rs                   # Main library entry point
│   └── tests/                   # Integration and unit tests
├── .clinerules                  # Cline's rules and learned patterns
├── build.rs                     # Custom build script
├── Cargo.toml                   # Workspace and package definitions
└── README.md                    # This file
```

### Core Components

-   **`alkanes/factory`**: Implements the factory pattern for creating and managing AMM pools. It serves as a registry for all pools on the platform.
-   **`alkanes/pool`**: Contains the core logic for the AMM pools, including swapping, liquidity provision, and fee collection.
-   **`alkanes/oyl-token`**: An implementation of a standard token contract, used as the native `OYL` token.
-   **`alkanes/alkanes-runtime-*`**: These crates provide the necessary runtime support for the factory and pool contracts to operate within the Alkanes framework.
-   **`src/tests`**: A comprehensive test suite that covers all aspects of the AMM's functionality, ensuring correctness and security.

## Getting Started

To build the project, run:
```bash
cargo build
```

To run the tests, use:
```bash
cargo test
