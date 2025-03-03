# System Patterns: swap.oyl.io

## Architecture Overview

swap.oyl.io follows a modular architecture with clear separation of concerns. The system is built on the Alkanes metaprotocol and is organized into several key components that work together to provide a complete AMM solution.

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│     Router      │────▶│     Factory     │────▶│      Pool       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                       │                       │
        │                       │                       │
        ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Alkanes Metaprotocol                       │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Bitcoin Network                          │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Router (`AMMRouter`)
- **Responsibility**: Directs user interactions to the appropriate pools and handles multi-hop swaps.
- **Key Functions**:
  - Routing swap operations through multiple pools
  - Finding the optimal path for token swaps
  - Adding and removing liquidity through the appropriate pools
- **Design Pattern**: Facade pattern, providing a simplified interface to the complex system of pools.

### 2. Factory (`AMMFactory` / `OylAMMFactory`)
- **Responsibility**: Creates and manages liquidity pools.
- **Key Functions**:
  - Creating new pools for token pairs
  - Finding existing pools for token pairs
  - Initializing pool parameters
- **Design Pattern**: Factory pattern, centralizing the creation of pool instances.
- **Variants**:
  - `AMMFactory`: Standard factory implementation
  - `OylAMMFactory`: Extended implementation that creates additional pools with the OYL token and implements the buyback mechanism

### 3. Pool (`AMMPool` / `OylAMMPool`)
- **Responsibility**: Manages token reserves and executes swap operations.
- **Key Functions**:
  - Maintaining token reserves
  - Executing swaps based on the constant product formula (x * y = k)
  - Minting and burning LP tokens
  - Calculating swap outputs
- **Design Pattern**: Composite pattern with delegation, allowing for extension of functionality.
- **Variants**:
  - `AMMPool`: Standard pool implementation
  - `OylAMMPool`: Extended implementation that redirects a portion of fees to buy back OYL tokens

## Design Patterns

### 1. Delegate Pattern
The system extensively uses the delegate pattern to allow for extension of functionality. Both the `AMMPool` and `AMMFactory` classes have a delegate field that can be set to override default behavior.

```rust
pub struct AMMPool {
    delegate: Option<Box<dyn AMMPoolBase>>,
}

impl AMMPool {
    pub fn set_delegate(&mut self, delegate: Box<dyn AMMPoolBase>) {
        self.delegate = Some(delegate);
    }
}
```

This pattern is used to implement the OYL-specific functionality without modifying the base implementation.

### 2. Trait-Based Composition
The system uses traits to define interfaces and compose functionality. For example, the `AMMPoolBase` trait defines the core functionality of a pool, and the `AMMReserves` trait provides a default implementation for managing reserves.

```rust
pub trait AMMPoolBase {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer);
    fn mint(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse>;
    fn burn(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse>;
    fn swap(&self, parcel: AlkaneTransferParcel, amount_out_predicate: u128) -> Result<CallResponse>;
    // ...
}

pub trait AMMReserves: AlkaneResponder + AMMPoolBase {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        // Default implementation
    }
}
```

### 3. Zero-Sum Logic
The system enforces zero-sum logic to ensure that tokens cannot be created or destroyed within the system. This is implemented through careful accounting of token transfers and reserves.

```rust
fn swap(&self, parcel: AlkaneTransferParcel, amount_out_predicate: u128) -> Result<CallResponse> {
    // Ensure only one token is being swapped
    if parcel.0.len() != 1 {
        return Err(anyhow!(format!("payload can only include 1 alkane, sent {}", parcel.0.len())));
    }
    
    // Calculate output based on constant product formula
    let transfer = parcel.0[0].clone();
    let (previous_a, previous_b) = self.previous_reserves(&parcel);
    let output = if &transfer.id == &reserve_a.id {
        AlkaneTransfer {
            id: reserve_b.id,
            value: self.get_amount_out(transfer.value, previous_a.value, previous_b.value)?,
        }
    } else {
        AlkaneTransfer {
            id: reserve_a.id,
            value: self.get_amount_out(transfer.value, previous_b.value, previous_a.value)?,
        }
    };
    
    // Return the calculated output
    let mut response = CallResponse::default();
    response.alkanes = AlkaneTransferParcel(vec![output]);
    Ok(response)
}
```

### 4. Constant Product Formula
The AMM uses the constant product formula (x * y = k) to determine swap prices, ensuring that the product of the reserves remains constant after each swap (minus fees).

```rust
fn get_amount_out(&self, amount: u128, reserve_from: u128, reserve_to: u128) -> Result<u128> {
    let amount_in_with_fee = U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount);
    let numerator = amount_in_with_fee * U256::from(reserve_to);
    let denominator = U256::from(1000) * U256::from(reserve_from) + amount_in_with_fee;
    Ok((numerator / denominator).try_into()?)
}
```

## Data Flow

### Pool Creation
1. User sends two tokens to the Factory
2. Factory creates a new Pool for the token pair
3. Pool initializes with the provided tokens as reserves
4. Factory returns LP tokens to the user

### Swapping
1. User sends tokens to the Router with a specified path
2. Router identifies the appropriate pools for each hop in the path
3. Router executes the swap through each pool
4. Pools calculate the output amount based on the constant product formula
5. Router returns the final output tokens to the user

### Adding Liquidity
1. User sends tokens to the Router
2. Router identifies the appropriate pool
3. Pool calculates the LP tokens to mint based on the provided liquidity
4. Pool mints LP tokens and sends them to the user

### Removing Liquidity
1. User sends LP tokens to the Router
2. Router identifies the appropriate pool
3. Pool burns the LP tokens and calculates the tokens to return
4. Pool sends the underlying tokens back to the user

## Security Considerations

### Reentrancy Protection
The system is designed to prevent reentrancy attacks by completing all state changes before making external calls.

### Integer Overflow Protection
The system uses checked arithmetic operations to prevent integer overflows.

```rust
overflow_error(total_supply.checked_add(liquidity))?
```

### Minimum Liquidity
To prevent manipulation of pool prices, a minimum amount of liquidity (1000 wei) is permanently locked in each pool.

```rust
pub const MINIMUM_LIQUIDITY: u128 = 1000;
```

### Slippage Protection
Users can specify a minimum output amount to protect against slippage.

```rust
if output.value < amount_out_predicate {
    return Err(anyhow!("predicate failed: insufficient output"));
}
```

## Extension Points

The system is designed to be extensible in several ways:

1. **Custom Pool Implementations**: By implementing the `AMMPoolBase` trait, developers can create custom pool types with different pricing functions or fee structures.

2. **Custom Factory Implementations**: By extending the `AMMFactoryBase` trait, developers can create custom factory types that create specialized pools.

3. **Router Extensions**: The router can be extended to support additional operations or routing algorithms.

4. **Fee Redirection**: As demonstrated by the OylAMMPool implementation, fees can be redirected to support various tokenomics models.