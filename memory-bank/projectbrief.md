# Project Brief: swap.oyl.io

## Project Overview
swap.oyl.io is a decentralized exchange (DEX) library built on the Alkanes metaprotocol, leveraging Protorunes to enable smart contract functionality on the Bitcoin network. This project aims to create an automated market maker (AMM) protocol akin to Uniswap V2, supporting the seamless swapping of Protorunes with an invariant of a * b = k. The library is designed with zero-sum logic, ensuring that new tokens cannot be arbitrarily minted within pools. The system rewards liquidity providers (LPs) by taking a portion of swap fees, with the OylSwap variant also directing a portion of fees to buy back the OYL token.

## Key Components
- **Protorunes:** Tokens created by burning Runes, sent to smart contract IDs via a custom indexer.
- **Alkanes Protocol:** Underlying metaprotocol on Bitcoin, allowing smart contracts through ID-based transfers.
- **Metashrew Backend:** Powers the contract interactions and indexing.
- **AMM Logic:** Follows the x * y = k invariant, providing liquidity pool (LP) tokens to depositors and taking swap fees for LP rewards.

## Goals
- Build a robust and secure swapping protocol for Protorunes.
- Ensure zero-sum logic to prevent unauthorized token minting.
- Provide incentives for liquidity providers through fee distribution.
- Implement the OylSwap fee redirection to support the OYL token.

## Target Users
- Bitcoin developers and users seeking smart contract capabilities.
- Liquidity providers looking to earn rewards on their assets.
- Traders swapping between Protorunes and other assets.

## Expected Outcome
A fully functional library that allows seamless Protorune swaps on Bitcoin, with a secure, fair, and efficient market-making mechanism that aligns incentives for all participants.

