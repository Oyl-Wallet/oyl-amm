# Project Brief: swap.oyl.io

## Overview
swap.oyl.io is a decentralized exchange (DEX) implementation using the Automated Market Maker (AMM) model, similar to Uniswap but implemented for Bitcoin or a Bitcoin-related blockchain. The project provides contracts for generic AMM pools and the Oylswap platform.

## Core Requirements

1. **AMM Functionality**: Implement core AMM functionality including:
   - Pool creation and initialization
   - Liquidity provision and removal
   - Token swapping with various strategies
   - Fee collection

2. **Factory Pattern**: Implement a factory pattern for creating and managing AMM pools:
   - Create new pools
   - Find existing pools
   - Manage pool collections
   - Facilitate multi-hop swaps

3. **Token Implementation**: Provide a standard token implementation (OYL token) with:
   - Name, symbol, and total supply management
   - Minting capabilities
   - Standard token operations

4. **Testing**: Comprehensive test suite covering:
   - Pool initialization (normal, skewed, zero, bad)
   - Factory functionality (double init, one incoming, duplicate pool)
   - Liquidity operations (add liquidity, burn liquidity)
   - Swap operations (normal swap, large swap, swap with factory)
   - Pool information (name, details, finding pools)

## Goals

1. Create a robust and secure AMM implementation for Bitcoin-based blockchains
2. Provide efficient and gas-optimized contracts for token swapping
3. Ensure proper error handling and security measures
4. Support various token pairs and multi-hop swaps
5. Implement fee collection mechanisms for liquidity providers

## Scope

The project scope includes:
- Core AMM contracts (Pool, Factory)
- OYL token implementation
- Runtime support for the Alkanes framework
- Comprehensive test suite

The project does not include:
- Frontend implementation
- Deployment scripts
- Documentation website
- Governance mechanisms (could be added in future versions)