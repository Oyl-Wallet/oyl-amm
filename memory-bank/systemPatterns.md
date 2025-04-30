# Swap.oyl.io System Patterns

## System Architecture

Swap.oyl.io follows a modular architecture inspired by Uniswap's design, with several key components working together to provide a complete decentralized exchange solution on the Bitcoin blockchain using the ALKANES metaprotocol.

### Core Components

#### 1. Factory Contract

The Factory contract is responsible for creating and managing liquidity pools. It serves as a registry of all pools in the system and ensures that only one pool exists for each token pair.

**Key Responsibilities:**
- Creating new liquidity pools for token pairs
- Maintaining a registry of all created pools
- Providing lookup functions to find pools by token pair
- Ensuring uniqueness of pools (one pool per token pair)
- Tracking all pools for enumeration and discovery

**Implementation:**
- Defined in `alkanes/alkanes-runtime-factory/src/lib.rs`
- Implements the `AMMFactoryBase` trait
- Uses the `declare_alkane!` macro to create the WASM contract

#### 2. Pool Contract

The Pool contract implements the core AMM functionality, including liquidity provision, token swapping, and fee collection.

**Key Responsibilities:**
- Managing token reserves
- Implementing the constant product formula (x * y = k)
- Processing swaps between the two tokens
- Handling liquidity provision and removal
- Collecting and distributing trading fees

**Implementation:**
- Defined in `alkanes/alkanes-runtime-pool/src/lib.rs`
- Implements the `AMMPoolBase` trait
- Uses the `declare_alkane!` macro to create the WASM contract

#### 3. Factory Contract

The Factory contract provides high-level functions for users to interact with the system, including multi-hop swaps and optimized trading paths.

**Key Responsibilities:**
- Finding optimal trading paths between tokens
- Executing multi-hop swaps through multiple pools
- Handling complex liquidity operations
- Providing a simplified interface for users

**Implementation:**
- Defined in `alkanes/factory/src/lib.rs`
- Uses the Factory contract to find pools
- Implements the `AlkaneResponder` trait

#### 4. OYL-Specific Implementations

The OYL-specific implementations provide customized versions of the pool and factory contracts for the OYL token ecosystem.

**Key Responsibilities:**
- Implementing OYL-specific logic
- Customizing fee structures or other parameters
- Providing specialized functionality for the OYL ecosystem

**Implementation:**
- Defined in `alkanes/oyl-pool/src/lib.rs` and `alkanes/oyl-factory/src/lib.rs`
- Extend the base pool and factory implementations

### Component Relationships

```
┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │
│  Factory         │────▶│   Factory       │
│  Contract       │     │   Contract      │
│                 │     │                 │
└─────────────────┘     └─────────────────┘
         │                       │
         │                       │ creates
         │                       ▼
         │              ┌─────────────────┐
         │              │                 │
         └─────────────▶│   Pool          │
                        │   Contract      │
                        │                 │
                        └─────────────────┘
```

1. **Factory → Factory**: The Factory contract calls the Factory contract to find pools for specific token pairs.
2. **Factory → Pool**: The Factory contract creates and manages Pool contracts.
3. **Factory → Pool**: The Factory contract interacts directly with Pool contracts for swaps and liquidity operations.

## Key Design Patterns

### 1. Factory Pattern

The Factory pattern is used to create and manage pool instances. This pattern centralizes pool creation logic and ensures that only one pool exists for each token pair.

**Implementation:**
```rust
impl AMMFactoryBase for AMMFactory {
    fn create_new_pool(&self, context: Context) -> Result<CallResponse> {
        // Check that there are exactly two tokens
        if context.incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("must send two runes to initialize a pool"));
        }
        
        // Sort the tokens to ensure consistent pool addresses
        let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
        let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
        
        // Get the next sequence number for the pool
        let next_sequence = self.sequence();
        
        // Store the pool address in the registry
        self.pool_pointer(&a, &b)
            .set(Arc::new(AlkaneId::new(2, next_sequence).into()));
        
        // Create the pool
        self.call(
            &Cellpack {
                target: AlkaneId {
                    block: 6,
                    tx: self.pool_id()?,
                },
                inputs: vec![0, a.block, a.tx, b.block, b.tx],
            },
            &AlkaneTransferParcel(vec![
                context.incoming_alkanes.0[0].clone(),
                context.incoming_alkanes.0[1].clone(),
            ]),
            self.fuel(),
        )
    }
}
```

### 2. Constant Product Formula

The Constant Product Formula (x * y = k) is the core mathematical model used for AMM pricing. It ensures that the product of the reserves remains constant after trades, creating a price curve that adjusts based on trade size.

**Implementation:**
```rust
fn get_amount_out(&self, amount: u128, reserve_from: u128, reserve_to: u128) -> Result<u128> {
    let amount_in_with_fee =
        U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount);
    let numerator = amount_in_with_fee * U256::from(reserve_to);
    let denominator = U256::from(1000) * U256::from(reserve_from) + amount_in_with_fee;
    Ok((numerator / denominator).try_into()?)
}
```

### 3. Delegate Pattern

The Delegate pattern is used for runtime behavior customization. This allows for different implementations of the same interface to be used interchangeably.

**Implementation:**
```rust
pub struct AMMPool {
    delegate: Option<Box<dyn AMMPoolBase>>,
}

impl AMMPool {
    pub fn default() -> Self {
        let mut pool = AMMPool { delegate: None };
        pool.set_delegate(Box::new(pool.clone()));
        pool
    }

    pub fn set_delegate(&mut self, delegate: Box<dyn AMMPoolBase>) {
        self.delegate = Some(delegate);
    }
}
```

### 4. Trait-based Polymorphism

Trait-based polymorphism is used to define interfaces for different components, allowing for multiple implementations of the same interface.

**Implementation:**
```rust
pub trait AMMPoolBase {
    fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        context: Context
    ) -> Result<CallResponse>;
    
    fn mint(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse>;
    
    fn burn(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse>;
    
    fn swap(
        &self,
        parcel: AlkaneTransferParcel,
        amount_out_predicate: u128
    ) -> Result<CallResponse>;
    
    // Other methods...
}
```

### 5. Command Pattern

The Command pattern is used to encapsulate operations as objects. In the ALKANES system, this is implemented through opcodes that determine which function to execute.

**Implementation:**
```rust
impl AlkaneResponder for AMMPool {
    fn execute(&self) -> Result<CallResponse> {
        if let Some(delegate) = &self.delegate {
            let context = self.context()?;
            let mut inputs = context.inputs.clone();
            match shift_or_err(&mut inputs)? {
                0 => delegate.process_inputs_and_init_pool(inputs, context),
                1 => delegate.mint(context.myself, context.incoming_alkanes),
                2 => delegate.burn(context.myself, context.incoming_alkanes),
                3 => delegate.swap(context.incoming_alkanes, shift_or_err(&mut inputs)?),
                5 => delegate.pool_details(),
                // Other opcodes...
                _ => Err(anyhow!("unrecognized opcode")),
            }
        } else {
            Err(anyhow!("No delegate set"))
        }
    }
}
```

## Data Flow Patterns

### 1. Pool Creation Flow

```
1. User calls Factory or Factory to create a new pool
2. Factory checks if pool already exists for the token pair
3. If not, Factory creates a new Pool contract
4. Factory registers the new pool in its registry
5. Factory initializes the pool with the token pair
6. Pool initializes its state and mints LP tokens to the creator
```

#### Detailed Pool Creation Process

The pool creation process involves several critical steps that must be executed in the correct order:

1. **Factory Initialization Check**:
   - The factory checks if it has been initialized (`/initialized` storage key)
   - If not initialized, it fails with "No delegate set" error
   - If already initialized, it proceeds with pool creation

2. **Token Pair Validation**:
   - The factory validates that exactly two tokens are provided
   - If not exactly two tokens, it fails with "must send two runes to initialize a pool" error

3. **Token Pair Sorting**:
   - The factory sorts the token pair to ensure consistent pool addresses
   - This is done using the `sort_alkanes` function:
   ```rust
   fn sort_alkanes((a, b): (AlkaneId, AlkaneId)) -> (AlkaneId, AlkaneId) {
       if a < b {
           (a, b)
       } else {
           (b, a)
       }
   }
   ```

4. **Pool Registration**:
   - The factory registers the pool in two places:
     a. Token pair mapping: `/pools/{token1}/{token2}` → pool_id
     b. Sequential registry: `/all_pools/{index}` → pool_id
   - The factory also updates the total pool count: `/all_pools_length` → total + 1

5. **Pool Initialization**:
   - The factory calls the pool contract with the token pair
   - The pool initializes its state with the token reserves
   - The pool mints LP tokens to the creator

This process ensures that:
- Each token pair has exactly one pool
- Pools can be looked up by token pair
- All pools can be enumerated sequentially
- Pool creation is atomic (either succeeds completely or fails)

#### Common Issues in Pool Creation

During testing and development, we encountered several common issues:

1. **Multiple Factory Initialization**:
   - Attempting to initialize the factory multiple times
   - Results in "already initialized" error
   - Solution: Initialize the factory only once per test

2. **Incorrect Token Count**:
   - Providing fewer or more than two tokens
   - Results in "must send two runes to initialize a pool" error
   - Solution: Always provide exactly two tokens

3. **Missing Pool Registration**:
   - Failing to register the pool in the sequential registry
   - Results in pools not being discoverable via "get all pools"
   - Solution: Ensure both registration steps are completed

4. **Storage Conflicts**:
   - Using the same storage keys in different tests
   - Results in "already holds a binary" errors
   - Solution: Clear storage between tests using the `clear()` function

### 2. Swap Flow

```
1. User calls Factory to swap tokens
2. Factory finds the optimal path for the swap
3. Factory calls the appropriate Pool(s) to execute the swap
4. Pool calculates the output amount using the constant product formula
5. Pool transfers the output tokens to the user
6. Pool updates its reserves
```

### 3. Liquidity Provision Flow

```
1. User calls Factory or Pool to add liquidity
2. Pool calculates the appropriate token ratio based on current reserves
3. Pool accepts the tokens and adds them to reserves
4. Pool mints LP tokens to the user proportional to their contribution
5. Pool updates its total supply of LP tokens
```

### 4. Liquidity Removal Flow

```
1. User calls Factory or Pool to remove liquidity
2. Pool calculates the amount of tokens to return based on LP tokens
3. Pool burns the LP tokens
4. Pool transfers the underlying tokens to the user
5. Pool updates its reserves and total supply
```

## Error Handling Patterns

The system uses Rust's `anyhow` for error handling, providing rich error context and propagation.

**Implementation:**
```rust
fn execute(&self) -> Result<CallResponse> {
    match operation {
        // Normal operations...
        _ => Err(anyhow!("unrecognized opcode"))
    }
}
```

Common error handling patterns include:

1. **Validation Errors**: Checking inputs before processing
   ```rust
   if context.incoming_alkanes.0.len() != 2 {
       return Err(anyhow!("must send two runes to initialize a pool"));
   }
   ```

2. **State Validation**: Ensuring the contract is in the correct state
   ```rust
   let mut pointer = StoragePointer::from_keyword("/initialized");
   if pointer.get().len() == 0 {
       // Initialize
   } else {
       return Err(anyhow!("already initialized"));
   }
   ```

3. **Arithmetic Error Handling**: Safely handling arithmetic operations
   ```rust
   let amount_a = overflow_error(liquidity.checked_mul(reserve_a.value))? / total_supply;
   ```

## Storage Patterns

The system uses a key-value storage pattern for persistent state:

1. **Storage Pointers**: Used to access and modify state
   ```rust
   StoragePointer::from_keyword("/totalsupply").set_value::<u128>(v);
   ```

2. **Namespaced Storage**: Using prefixes to organize storage
   ```rust
   StoragePointer::from_keyword("/pools/")
       .select(&a.clone().into())
       .keyword("/")
       .select(&b.clone().into())
   ```

3. **Serialization**: Converting complex objects to bytes for storage
   ```rust
   pub fn try_to_vec(&self) -> Vec<u8> {
       let mut bytes = Vec::new();
       // Serialize fields...
       bytes
   }
   ```

## Testing Patterns

The system uses comprehensive testing patterns:

1. **Unit Tests**: Testing individual components
   ```rust
   #[test]
   fn test_get_amount_out() {
       // Test implementation...
   }
   ```

2. **Integration Tests**: Testing interactions between components
   ```rust
   #[wasm_bindgen_test]
   fn test_amm_pool_swap() -> Result<()> {
       // Test implementation...
   }
   ```

3. **Test Fixtures**: Reusable test setups
   ```rust
   fn test_amm_pool_init_fixture(amount1: u128, amount2: u128, use_oyl: bool) -> Result<(Block, AmmTestDeploymentIds)> {
       // Test fixture implementation...
   }
   ```

## "Get All Pools" Functionality

The "get all pools" functionality follows the pattern used in Uniswap and similar AMM protocols. It involves:

1. **Pool Tracking**: The Factory contract tracks all created pools in an array-like structure.
2. **Pool Enumeration**: The Factory provides methods to get the total number of pools and retrieve pools by index.
3. **Pool Details**: The Factory can retrieve details for each pool by calling the pool's `pool_details` method.

This functionality will be implemented by extending the `AMMFactoryBase` trait with new methods:

1. **all_pools_length**: Returns the total number of pools
2. **all_pools**: Returns a pool by index
3. **get_all_pools**: Returns a list of all pool IDs
4. **get_all_pools_details**: Returns details for all pools

The implementation will use the following storage structure:

1. `/all_pools_length`: Stores the total number of pools
2. `/all_pools/{index}`: Stores the pool ID at the given index

When a new pool is created, it will be added to this registry, allowing for efficient enumeration of all pools.

## ALKANES Trace Data Pattern

The ALKANES metaprotocol uses a specific pattern for returning data from contract functions, which is essential to understand for implementing and testing view functions:

### 1. Trace Data Structure

The trace data structure in ALKANES follows a consistent pattern:

```
┌─────────────────────┐
│ Transaction Header  │
├─────────────────────┤
│ Execution Context   │
├─────────────────────┤
│ Call Response       │
├─────────────────────┤
│ Return Data         │ ← This is where the actual function return data is located
└─────────────────────┘
```

1. **Transaction Header**: Contains metadata about the transaction
2. **Execution Context**: Contains information about the execution environment
3. **Call Response**: Contains the response from the contract call
4. **Return Data**: Contains the actual data returned by the function

### 2. Accessing Trace Data

To access trace data from a contract function call:

1. Use the `view::trace()` function with the transaction outpoint
2. Access `vout 3` of the transaction (not `vout 0`)
3. The trace data will be returned as a byte array
4. The actual return data will be located at an offset within this byte array

### 3. Parsing Return Data

The return data format varies by function but typically follows these patterns:

1. **Count/Length Prefix**: Many functions start with a count or length (u128, 16 bytes)
2. **Fixed-Size Elements**: Elements of known size (e.g., AlkaneIds, each 32 bytes)
3. **Variable-Length Data**: Data with length prefixes for variable-sized elements
4. **Nested Structures**: Complex data may have nested structures with their own formats

### 4. Common Parsing Techniques

```rust
// Example: Parsing a collection of AlkaneIds from trace data
fn parse_alkane_ids(trace_data: &[u8], offset: usize) -> Result<Vec<AlkaneId>> {
    // Parse the count (first 16 bytes)
    let count_bytes: [u8; 16] = trace_data[offset..offset+16]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read count"))?;
    let count = u128::from_le_bytes(count_bytes);
    
    // Parse each AlkaneId
    let mut ids = Vec::new();
    for i in 0..count {
        let item_offset = offset + 16 + (i as usize * 32); // 16 bytes for count, 32 bytes per ID
        
        // Read block ID (16 bytes)
        let block_bytes: [u8; 16] = trace_data[item_offset..item_offset+16]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read block ID"))?;
        let block = u128::from_le_bytes(block_bytes);
        
        // Read tx ID (16 bytes)
        let tx_bytes: [u8; 16] = trace_data[item_offset+16..item_offset+32]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read tx ID"))?;
        let tx = u128::from_le_bytes(tx_bytes);
        
        ids.push(AlkaneId::new(block, tx));
    }
    
    Ok(ids)
}
```

### 5. Testing View Functions

When testing view functions that return data through the trace mechanism:

1. **Setup**: Create and initialize the necessary contracts
2. **Execute**: Call the view function with appropriate parameters
3. **Access**: Get the trace data from `vout 3` of the transaction
4. **Parse**: Parse the return data using knowledge of its structure
5. **Validate**: Verify that the parsed data matches expectations

This pattern is essential for implementing and testing any view function in the ALKANES system, including the "get all pools" functionality in Swap.oyl.io.

## Conclusion

The Swap.oyl.io system architecture follows established design patterns from the DeFi space, particularly Uniswap, while adapting them to the ALKANES metaprotocol on Bitcoin. The modular design with Factory, Pool, and Factory contracts provides a flexible and extensible foundation for decentralized trading.

Understanding the ALKANES trace data pattern is crucial for implementing and testing view functions like "get all pools", which enhance the system by providing ways to discover and interact with the available liquidity pools.
