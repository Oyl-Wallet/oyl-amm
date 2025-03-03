# Product Context: swap.oyl.io

## Problem Statement
Bitcoin, while being the most secure and widely adopted cryptocurrency, has traditionally lacked the smart contract capabilities found in platforms like Ethereum. This limitation has prevented the development of decentralized applications (dApps) and decentralized finance (DeFi) protocols directly on Bitcoin. The Alkanes metaprotocol and Protorunes aim to solve this by enabling smart contract functionality on Bitcoin, but there's still a need for fundamental DeFi primitives like token swapping.

## Solution
swap.oyl.io provides a decentralized exchange (DEX) library built on the Alkanes metaprotocol, enabling seamless swapping of Protorunes on the Bitcoin network. By implementing an automated market maker (AMM) protocol similar to Uniswap V2, it allows users to:

1. Create liquidity pools for any pair of Protorunes
2. Provide liquidity to earn fees
3. Swap between different Protorunes with minimal slippage
4. Route complex trades through multiple pools

The OylSwap variant adds an additional feature where a portion of the swap fees is used to buy back the OYL token, creating a value accrual mechanism for the protocol's native token.

## User Experience Goals

### For Traders
- **Seamless Swapping**: Users should be able to easily swap between different Protorunes with minimal friction.
- **Optimal Pricing**: The AMM should provide fair and efficient pricing based on the constant product formula (x * y = k).
- **Multi-hop Trades**: Complex trades involving multiple token pairs should be automatically routed for optimal execution.
- **Predictable Fees**: A fixed fee structure (0.4% per swap) ensures transparency and predictability.

### For Liquidity Providers
- **Passive Income**: LPs earn a share of the 0.4% fee charged on all swaps proportional to their share of the pool.
- **Fair Distribution**: LP tokens accurately represent a provider's share of the pool.
- **Minimal Impermanent Loss**: While impermanent loss is inherent to AMMs, the design aims to minimize its impact through efficient pricing.
- **Easy Liquidity Management**: Adding and removing liquidity should be straightforward processes.

### For Developers
- **Modular Design**: The library is designed to be modular, allowing developers to integrate it into their own applications.
- **Extensibility**: The architecture supports extending the base functionality, as demonstrated by the OylSwap variant.
- **Robust Testing**: Comprehensive test coverage ensures reliability and correctness.

## Key Differentiators

1. **Bitcoin Native**: Unlike most DEXes that operate on Ethereum or other smart contract platforms, swap.oyl.io is built specifically for Bitcoin using the Alkanes metaprotocol.

2. **Zero-Sum Logic**: The protocol ensures that new tokens cannot be arbitrarily minted within pools, maintaining the integrity of the token supply.

3. **OYL Token Buyback**: The OylSwap variant directs a portion of swap fees to buy back the OYL token, creating a sustainable value accrual mechanism.

4. **Protorune Compatibility**: Specifically designed to work with Protorunes, which are tokens created by burning Runes and sent to smart contract IDs.

## Target Market
The primary users of swap.oyl.io will be:

1. **Bitcoin Enthusiasts**: Users who prefer to stay within the Bitcoin ecosystem but want access to DeFi functionality.
2. **Protorune Holders**: Those who have invested in various Protorunes and need a way to trade between them.
3. **Yield Seekers**: Investors looking to earn passive income by providing liquidity to pools.
4. **Bitcoin DApp Developers**: Developers building on the Alkanes metaprotocol who need a reliable token swapping mechanism.

## Success Metrics
The success of swap.oyl.io can be measured by:

1. **Total Value Locked (TVL)**: The amount of assets locked in liquidity pools.
2. **Trading Volume**: The total value of swaps executed through the protocol.
3. **Number of Unique Pools**: Indicates the diversity of trading pairs available.
4. **Protocol Revenue**: Fees generated for liquidity providers and the OYL token buyback.
5. **Developer Adoption**: Integration of the library into other Bitcoin DApps.