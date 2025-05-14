# System Patterns: swap.oyl.io

## System Architecture

The swap.oyl.io system is built on a modular architecture using the Alkanes framework, which appears to be a custom blockchain framework for Bitcoin or Bitcoin-related blockchains. The system consists of several key components:

```
┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │
│   AMM Factory   │────▶│    AMM Pools    │
│                 │     │                 │
└─────────────────┘     └─────────────────┘
        │                       │
        │                       │
        ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │
│   OYL Token     │     │  Other Tokens   │
│                 │     │                 │
└─────────────────┘     └─────────────────┘
```

1. **AMM Factory**: Central component that creates and manages pools
2. **AMM Pools**: Individual liquidity pools for token pairs
3. **OYL Token**: Native token of the platform
4. **Runtime Support**: Alkanes runtime components for pools and factory

## Key Technical Decisions

1. **Alkanes Framework**: The project uses the Alkanes framework, which appears to be a custom blockchain framework for Bitcoin or Bitcoin-related blockchains. This enables complex smart contract functionality on Bitcoin, which traditionally has limited scripting capabilities.

2. **Constant Product Formula**: The AMM uses the constant product formula (x * y = k) for determining exchange rates, similar to Uniswap v2. This formula ensures that the product of the quantities of the two tokens in a pool remains constant during swaps.

3. **Factory Pattern**: The project uses a factory pattern for creating and managing pools, which centralizes pool creation logic and makes it easier to discover existing pools.

4. **Message-Based Architecture**: The system uses a message-based architecture where operations are defined as messages that are dispatched to the appropriate handler.

5. **Rust Implementation**: The entire system is implemented in Rust, which provides memory safety and performance benefits.

## Design Patterns in Use

1. **Factory Pattern**: The AMM Factory creates and manages pools, centralizing the creation logic and providing a registry of all pools.

2. **Message Dispatch Pattern**: Operations are defined as messages that are dispatched to the appropriate handler using the `MessageDispatch` trait.

3. **Authenticated Responder Pattern**: Some components use the `AuthenticatedResponder` trait to ensure that only authorized users can perform certain operations.

4. **Mintable Token Pattern**: The `MintableToken` trait provides standard functionality for tokens that can be minted.

5. **Constant Product AMM Pattern**: The AMM uses the constant product formula (x * y = k) for determining exchange rates, which ensures that the product of the quantities of the two tokens in a pool remains constant during swaps.

## Component Relationships

### AMM Factory

The AMM Factory is responsible for:
- Creating new pools
- Finding existing pools
- Managing the registry of all pools
- Facilitating multi-hop swaps

It interacts with:
- AMM Pools: Creates and manages pools
- Tokens: Interacts with tokens during pool creation and swaps

### AMM Pools

Each AMM Pool represents a liquidity pool for a pair of tokens and is responsible for:
- Holding token reserves
- Executing swaps between the two tokens
- Managing liquidity (adding and removing)
- Collecting and distributing fees

It interacts with:
- AMM Factory: Created by and registered with the factory
- Tokens: Holds token reserves and executes token transfers

### OYL Token

The OYL Token is the native token of the platform and provides:
- Standard token functionality (name, symbol, total supply)
- Minting capabilities

## Critical Implementation Paths

1. **Pool Initialization**:
   ```
   User -> AMM Factory -> Create Pool -> Initialize Pool -> Mint LP Tokens -> User
   ```

2. **Adding Liquidity**:
   ```
   User -> AMM Pool -> Add Liquidity -> Calculate Shares -> Mint LP Tokens -> User
   ```

3. **Swapping Tokens**:
   ```
   User -> AMM Pool -> Swap -> Calculate Exchange Rate -> Transfer Tokens -> User
   ```

4. **Multi-hop Swaps**:
   ```
   User -> AMM Factory -> Swap Along Path -> Pool 1 -> Pool 2 -> ... -> Pool N -> User
   ```

5. **Removing Liquidity**:
   ```
   User -> AMM Pool -> Burn LP Tokens -> Calculate Share -> Transfer Tokens -> User