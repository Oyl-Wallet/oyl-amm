# Active Context: swap.oyl.io

## Current Focus

The current focus of the swap.oyl.io project is on implementing and testing the core AMM functionality. The project has established the fundamental components of an AMM system:

1. **Router**: For directing user interactions and handling multi-hop swaps
2. **Factory**: For creating and managing liquidity pools
3. **Pool**: For executing swaps and managing liquidity

The implementation includes both standard and OYL-specific variants, with the latter incorporating the OYL token buyback mechanism.

## Recent Changes

Recent development has focused on:

1. **Testing**: Implementing comprehensive test cases for all aspects of the AMM functionality, including fixing the OYL pool swap test to properly verify the OYL token buyback mechanism
2. **OYL Integration**: Extending the base AMM implementation to support the OYL token buyback mechanism
3. **Router Optimization**: Improving the routing algorithm for multi-hop swaps

## Active Decisions

### 1. Fee Structure
- **Decision**: Implement a fixed 0.4% fee on all swaps (DEFAULT_FEE_AMOUNT_PER_1000 = 4)
- **Rationale**: This fee structure balances the need for liquidity provider incentives with competitive swap rates
- **Status**: Implemented and tested
- **Considerations**: May need to be adjusted based on market conditions and user feedback

### 2. OYL Token Buyback Mechanism
- **Decision**: Implement a mechanism where a portion of swap fees is used to buy back the OYL token
- **Rationale**: Creates a value accrual mechanism for the OYL token
- **Status**: Basic implementation in place, needs further refinement
- **Considerations**: The optimal portion of fees to redirect to buybacks needs to be determined

### 3. Multi-hop Routing
- **Decision**: Implement a simple path-based routing algorithm for multi-hop swaps
- **Rationale**: Enables complex trades through multiple pools
- **Status**: Implemented and tested
- **Considerations**: More sophisticated routing algorithms could be implemented in the future

### 4. Minimum Liquidity
- **Decision**: Lock a minimum amount of liquidity (1000 wei) in each pool
- **Rationale**: Prevents manipulation of pool prices and ensures a minimum level of liquidity
- **Status**: Implemented and tested
- **Considerations**: The optimal minimum liquidity amount may need to be adjusted

## Current Challenges

### 1. Gas Efficiency
- **Challenge**: Optimizing the implementation for gas efficiency
- **Approach**: Identify and optimize gas-intensive operations
- **Status**: Ongoing
- **Next Steps**: Profile gas usage and identify optimization opportunities

### 2. Price Impact
- **Challenge**: Minimizing price impact for large swaps
- **Approach**: Implement better routing algorithms and consider concentrated liquidity
- **Status**: Under consideration
- **Next Steps**: Research and prototype improved routing algorithms

### 3. Impermanent Loss
- **Challenge**: Addressing impermanent loss for liquidity providers
- **Approach**: Explore mechanisms to mitigate impermanent loss
- **Status**: Research phase
- **Next Steps**: Evaluate potential solutions and their trade-offs

## Upcoming Work

### Short-term (Next 2 Weeks)
1. **Refine OYL Buyback Mechanism**: Optimize the portion of fees redirected to buybacks
2. **Improve Test Coverage**: Add more edge cases and stress tests
3. **Documentation**: Enhance code documentation and developer guides

### Medium-term (Next 1-2 Months)
1. **Advanced Routing**: Implement more sophisticated routing algorithms
2. **Performance Optimization**: Identify and address performance bottlenecks
3. **Integration Testing**: Test integration with other Alkanes-based protocols

### Long-term (3+ Months)
1. **Concentrated Liquidity**: Explore implementing concentrated liquidity similar to Uniswap V3
2. **Governance Mechanism**: Implement a governance system for protocol parameters
3. **Cross-chain Integration**: Explore bridging to other blockchains

## Open Questions

1. **Optimal Fee Structure**: Is the current 0.4% fee optimal for all token pairs, or should it be variable?
2. **Routing Efficiency**: How can we improve the routing algorithm to minimize slippage for complex trades?
3. **OYL Tokenomics**: What is the optimal portion of fees to redirect to OYL buybacks?
4. **Scalability**: How will the system perform under high load, and what optimizations can be made?
5. **Security**: Are there any potential vulnerabilities or attack vectors that need to be addressed?

## Current Priorities

1. **Robustness**: Ensure the core functionality is robust and well-tested
2. **OYL Integration**: Refine the OYL token buyback mechanism
3. **Documentation**: Improve documentation for developers and users
4. **Performance**: Optimize for gas efficiency and throughput