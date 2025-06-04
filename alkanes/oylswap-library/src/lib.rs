use alkanes_runtime::storage::StoragePointer;

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{id::AlkaneId, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::{byte_view::ByteView, index_pointer::KeyValuePointer};
use ruint::Uint;

pub const DEFAULT_FEE_AMOUNT_PER_1000: u128 = 5;

pub type U256 = Uint<256, 4>;

// Create a storage wrapper for U256 to implement ByteView
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StorableU256(pub U256);

impl ByteView for StorableU256 {
    fn from_bytes(v: Vec<u8>) -> Self {
        assert!(v.len() == 32, "Expected a byte vector of length 32.");
        // Convert bytes to U256 using from_le_bytes
        let mut bytes_array = [0u8; 32];
        bytes_array.copy_from_slice(&v);
        StorableU256(U256::from_le_bytes(bytes_array))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes::<32>().to_vec()
    }

    fn maximum() -> Self {
        StorableU256(U256::MAX)
    }

    fn zero() -> Self {
        StorableU256(U256::ZERO)
    }
}

impl From<U256> for StorableU256 {
    fn from(value: U256) -> Self {
        StorableU256(value)
    }
}

impl From<StorableU256> for U256 {
    fn from(value: StorableU256) -> Self {
        value.0
    }
}
pub struct Lock {}
impl Lock {
    pub fn lock_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/lock")
    }
    pub fn get_lock() -> u128 {
        Lock::lock_pointer().get_value::<u128>()
    }
    pub fn set_lock(v: u128) {
        Lock::lock_pointer().set_value::<u128>(v);
    }
    pub fn lock<F>(func: F) -> Result<CallResponse>
    where
        F: FnOnce() -> Result<CallResponse>,
    {
        if Lock::lock_pointer().get().len() != 0 && Lock::get_lock() == 1 {
            return Err(anyhow!("LOCKED"));
        }

        // Locking the resource
        Lock::set_lock(1);

        // Run the function (this is the _ in Solidity)
        let ret = func();

        // Unlocking after function execution
        Lock::set_lock(0);

        ret
    }
}

#[derive(Default)]
pub struct PoolInfo {
    pub token_a: AlkaneId,
    pub token_b: AlkaneId,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub pool_name: String,
}

impl PoolInfo {
    pub fn try_to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // token_a: 32 bytes (16 bytes block + 16 bytes tx)
        bytes.extend_from_slice(&self.token_a.block.to_le_bytes());
        bytes.extend_from_slice(&self.token_a.tx.to_le_bytes());

        // token_b: 32 bytes (16 bytes block + 16 bytes tx)
        bytes.extend_from_slice(&self.token_b.block.to_le_bytes());
        bytes.extend_from_slice(&self.token_b.tx.to_le_bytes());

        // reserves and total_supply: 16 bytes each
        bytes.extend_from_slice(&self.reserve_a.to_le_bytes());
        bytes.extend_from_slice(&self.reserve_b.to_le_bytes());
        bytes.extend_from_slice(&self.total_supply.to_le_bytes());

        // Add the pool name
        let name_bytes = self.pool_name.as_bytes();
        // Add the length of the name as a u32
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        // Add the name bytes
        bytes.extend_from_slice(name_bytes);

        bytes
    }

    pub fn from_vec(bytes: &[u8]) -> Result<Self> {
        // Minimum size: 32 (token_a) + 32 (token_b) + 16 (reserve_a) + 16 (reserve_b) + 16 (total_supply) + 4 (name_length) = 116 bytes
        if bytes.len() < 116 {
            return Err(anyhow!("Invalid bytes length for PoolInfo"));
        }

        let mut offset = 0;

        // Read token_a (32 bytes: 16 bytes block + 16 bytes tx)
        let token_a_block = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;
        let token_a_tx = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;

        // Read token_b (32 bytes: 16 bytes block + 16 bytes tx)
        let token_b_block = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;
        let token_b_tx = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;

        // Read reserve_a (16 bytes for u128)
        let reserve_a = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;

        // Read reserve_b (16 bytes for u128)
        let reserve_b = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;

        // Read total_supply (16 bytes for u128)
        let total_supply = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);
        offset += 16;

        // Read pool_name length (4 bytes for u32)
        let name_length = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?) as usize;
        offset += 4;

        // Check if we have enough bytes for the name
        if bytes.len() < offset + name_length {
            return Err(anyhow!("Invalid bytes length for pool_name"));
        }

        // Read pool_name
        let pool_name = String::from_utf8(bytes[offset..offset + name_length].to_vec())?;

        Ok(PoolInfo {
            token_a: AlkaneId {
                block: token_a_block,
                tx: token_a_tx,
            },
            token_b: AlkaneId {
                block: token_b_block,
                tx: token_b_tx,
            },
            reserve_a,
            reserve_b,
            total_supply,
            pool_name,
        })
    }
}

pub fn get_amount_out(amount_in: u128, reserve_in: u128, reserve_out: u128) -> Result<u128> {
    let amount_in_with_fee = U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount_in);

    let numerator = amount_in_with_fee * U256::from(reserve_out);
    let denominator = U256::from(1000) * U256::from(reserve_in) + amount_in_with_fee;
    Ok((numerator / denominator).try_into()?)
}

pub fn get_amount_in(amount_out: u128, reserve_in: u128, reserve_out: u128) -> Result<u128> {
    if amount_out == 0 {
        return Err(anyhow!("INSUFFICIENT_OUTPUT_AMOUNT"));
    }
    if reserve_in == 0 || reserve_out == 0 {
        return Err(anyhow!("INSUFFICIENT_LIQUIDITY"));
    }
    let numerator = U256::from(1000) * U256::from(reserve_in) * U256::from(amount_out);
    let denominator =
        U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(reserve_out - amount_out);
    Ok((numerator / denominator + U256::from(1)).try_into()?)
}

pub fn sort_alkanes((a, b): (AlkaneId, AlkaneId)) -> (AlkaneId, AlkaneId) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
