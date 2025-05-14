# Product Context: swap.oyl.io

## Why This Project Exists

The swap.oyl.io project exists to bring decentralized exchange (DEX) functionality to Bitcoin-based blockchains using the Automated Market Maker (AMM) model. While AMMs like Uniswap have become standard on Ethereum and other smart contract platforms, Bitcoin has traditionally lacked these capabilities due to its limited scripting language.

This project leverages the Alkanes framework to implement AMM functionality on Bitcoin or Bitcoin-related blockchains, enabling users to:
- Swap tokens without relying on centralized exchanges
- Provide liquidity and earn fees
- Create markets for new token pairs
- Perform multi-hop swaps across different token pairs

## Problems It Solves

1. **Centralization Risk**: Traditional exchanges require users to trust a central authority with their funds, creating security risks and single points of failure. swap.oyl.io eliminates this risk by enabling trustless, on-chain trading.

2. **Limited Bitcoin Functionality**: Bitcoin's scripting language is intentionally limited, making complex applications like AMMs challenging to implement. This project uses the Alkanes framework to overcome these limitations.

3. **Liquidity Fragmentation**: In traditional markets, liquidity is often fragmented across multiple venues. The AMM model aggregates liquidity into pools, making it more efficient for all participants.

4. **Price Discovery**: New tokens often struggle with price discovery. The AMM model provides an automatic pricing mechanism based on the ratio of tokens in each pool.

5. **Accessibility**: Traditional finance often has high barriers to entry. swap.oyl.io allows anyone to participate as a trader or liquidity provider without permission.

## How It Should Work

### Core Mechanics

1. **Pool Creation**:
   - Users can create new liquidity pools for any pair of tokens
   - Each pool maintains a balance of two tokens
   - The product of the token quantities (x * y = k) remains constant during swaps

2. **Liquidity Provision**:
   - Users can add liquidity by depositing both tokens in the correct ratio
   - Liquidity providers receive LP tokens representing their share of the pool
   - LP tokens can be redeemed later to withdraw liquidity plus accumulated fees

3. **Token Swapping**:
   - Users can swap one token for another using the pools
   - The exchange rate is determined by the ratio of tokens in the pool
   - A small fee is taken from each swap and distributed to liquidity providers
   - Multi-hop swaps allow trading between tokens that don't have a direct pool

4. **Factory Management**:
   - The factory contract manages the creation and tracking of all pools
   - It provides functions to find existing pools and facilitate multi-hop swaps

### User Flow

1. User connects their Bitcoin wallet to the swap.oyl.io interface
2. User selects tokens to swap or provide liquidity for
3. User approves the transaction and signs it with their wallet
4. The transaction is submitted to the Bitcoin network
5. Once confirmed, the swap or liquidity operation is executed on-chain
6. User receives their tokens or LP tokens in their wallet

## User Experience Goals

1. **Simplicity**: Make decentralized trading as simple as possible, even for users new to DeFi
2. **Transparency**: Provide clear information about exchange rates, fees, and slippage
3. **Security**: Ensure user funds are always secure and transactions are reliable
4. **Efficiency**: Minimize transaction costs and maximize capital efficiency
5. **Accessibility**: Make the platform accessible to users with varying levels of technical knowledge
6. **Reliability**: Ensure the system works consistently and predictably under all conditions