dno# OYL Protocol AMM Deployment Guide

This guide explains how to deploy the OYL Protocol AMM (Automated Market Maker) to any Bitcoin regtest environment using the deezel CLI tool.

## Overview

The OYL Protocol AMM is a decentralized exchange implementation similar to Uniswap, but built for Bitcoin using the Alkanes metaprotocol. The deployment script (`deploy-oyl-amm.sh`) automates the complete deployment process, including:

- Core AMM contracts (Factory, Pool logic)
- Token contracts (OYL token and test tokens)
- Infrastructure contracts (Beacon proxies, upgradeable contracts)
- Initial liquidity pools
- Verification of deployment

## Prerequisites

### 1. Deezel CLI Tool
Ensure the deezel CLI tool is installed and available in your PATH:
```bash
# Check if deezel is available
which deezel
deezel --help
```

### 2. Bitcoin and Sandshrew Nodes
You need access to:
- A Bitcoin node (for Bitcoin RPC operations)
- A Sandshrew/Metashrew node (for Alkanes operations)

For local development, you can use Docker to run these services.

### 3. Rust and WASM Target
The script will attempt to build contracts if needed:
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## Quick Start

### Basic Deployment to Local Regtest

```bash
# Deploy to local regtest environment
./deploy-oyl-amm.sh -p regtest --sandshrew-rpc-url http://localhost:18888
```

### Deployment with Auto-Confirmation

```bash
# Deploy without manual confirmation prompts
./deploy-oyl-amm.sh -p regtest --sandshrew-rpc-url http://localhost:18888 -y
```

### Custom Configuration

```bash
# Deploy with custom deezel binary
./deploy-oyl-amm.sh \
  -p regtest \
  --sandshrew-rpc-url http://localhost:18888 \
  --deezel-binary /data/deezel-old/target/release/deezel

# Deploy with custom settings
./deploy-oyl-amm.sh \
  -p regtest \
  --sandshrew-rpc-url http://localhost:18888 \
  --bitcoin-rpc-url http://user:pass@localhost:18443 \
  --deezel-binary /path/to/deezel \
  --wallet-file ~/.deezel/custom.json.asc \
  --fee-rate 5 \
  --passphrase mypassword
```

## Command Line Options

### Required Arguments

- `-p, --provider PROVIDER`: Network provider (regtest, testnet, signet, mainnet)
- `--sandshrew-rpc-url URL`: Sandshrew RPC endpoint URL

### Optional Arguments

- `--bitcoin-rpc-url URL`: Bitcoin RPC endpoint URL
- `--deezel-binary PATH`: Path to deezel binary (default: deezel)
- `--wallet-file PATH`: Path to wallet file (default: ~/.deezel/regtest.json.asc)
- `--passphrase PASS`: Wallet passphrase (default: testtesttest)
- `--fee-rate RATE`: Transaction fee rate in sat/vB (default: 1)
- `--no-mine`: Don't mine blocks after transactions
- `-y, --yes`: Auto-confirm all transactions
- `-h, --help`: Show help message

## Deployment Sequence

The script follows this deployment sequence, which matches the test suite implementation:

### Phase 1: Core Contracts
1. **AMM Pool Logic Implementation** - The template contract for all AMM pools
2. **Auth Token Factory** - Factory for creating authentication tokens
3. **AMM Factory Logic Implementation** - The core factory logic

### Phase 2: Token Contracts
4. **Test Token 1** - First test token with initial supply
5. **Test Token 2** - Second test token with initial supply  
6. **Test Token 3** - Third test token with initial supply
7. **OYL Token** - The native platform token

### Phase 3: Infrastructure
8. **Example Flashswap** - Example flashswap implementation
9. **Beacon Proxy** - Proxy contract for upgradeable pools
10. **Upgradeable Beacon** - Beacon for managing pool implementations

### Phase 4: Factory Setup
11. **Factory Proxy** - Upgradeable proxy for the factory
12. **Factory Initialization** - Initialize the factory with pool beacon

### Phase 5: Initial Pools
13. **Pool 1** - Token1/Token2 liquidity pool
14. **Pool 2** - Token2/Token3 liquidity pool

## Cellpack Notation

The script uses the Alkanes cellpack notation for deployments. The `[3, n]` notation means:

- `3`: The cellpack opcode for contract deployment
- `n`: The target transaction ID where the contract will be deployed
- Additional parameters: Constructor arguments for the contract

Examples:
```bash
# Deploy pool logic to transaction 3 with input 50
[3,3,50]

# Deploy token with initial mint
[3,5,0,1,1000000000000000000000]

# Initialize factory with beacon configuration
[3,1,0,0xbeac1,4,0xbeac0]
```

## Contract Addresses

After deployment, contracts will be available at these Alkane IDs:

| Contract | Block | TX | Description |
|----------|-------|----|-----------| 
| AMM Pool Logic | 4 | 3 | Template for all pools |
| Auth Token Factory | 4 | 4 | Authentication token factory |
| AMM Factory Logic | 4 | 2 | Factory implementation |
| Factory Proxy | 4 | 1 | Main factory interface |
| Token 1 | 4 | 3 | First test token |
| Token 2 | 4 | 5 | Second test token |
| Token 3 | 4 | 7 | Third test token |
| OYL Token | 4 | 9 | Platform token |
| Example Flashswap | 4 | 10 | Flashswap example |
| Beacon Proxy | 4 | 0xbeac1 | Pool proxy template |
| Upgradeable Beacon | 4 | 0xbeac0 | Pool beacon |

## Token Initial Supplies

The deployment creates tokens with these initial supplies:

- **Token 1**: 1,000,000,000,000,000,000,000 units
- **Token 2**: 2,000,000,000,000,000,000,000 units  
- **Token 3**: 1,000,000,000,000,000,000,000 units
- **OYL Token**: 1,000,000,000,000,000,000,000 units

## Verification

After deployment, the script performs verification:

1. **Wallet Balance Check** - Ensures wallet has remaining Bitcoin
2. **Token Balance Check** - Verifies token balances
3. **Factory Status** - Checks number of created pools
4. **Pool Verification** - Confirms pools are operational

## Troubleshooting

### Common Issues

1. **Deezel Not Found**
   ```
   Error: deezel command not found
   ```
   Solution: Install deezel and ensure it's in your PATH

2. **Connection Refused**
   ```
   Error: Connection refused to Sandshrew RPC
   ```
   Solution: Ensure Sandshrew node is running and accessible

3. **Insufficient Funds**
   ```
   Error: Insufficient funds for transaction
   ```
   Solution: The script automatically funds the wallet, but ensure Bitcoin node is running

4. **Contract Build Failures**
   ```
   Error: Contract file not found
   ```
   Solution: Ensure Rust and wasm32-unknown-unknown target are installed

### Debug Mode

For detailed debugging, run with debug logging:
```bash
RUST_LOG=debug ./deploy-oyl-amm.sh -p regtest --sandshrew-rpc-url http://localhost:18888
```

### Manual Verification

You can manually verify the deployment using deezel commands:

```bash
# Check factory status
deezel -p regtest --sandshrew-rpc-url http://localhost:18888 \
  --wallet-file ~/.deezel/regtest.json.asc --passphrase testtesttest \
  alkanes execute --inputs B:1000 --change [self:p2tr:2] --to [self:p2tr:1] \
  --fee-rate 1 -y '[3,1,4]:v0:v0'

# Check token balances
deezel -p regtest --sandshrew-rpc-url http://localhost:18888 \
  --wallet-file ~/.deezel/regtest.json.asc --passphrase testtesttest \
  alkanes balance --address [self:p2tr:0]
```

## Integration with Applications

After successful deployment, applications can interact with the AMM using:

- **Factory Proxy**: `AlkaneId { block: 4, tx: 1 }`
- **Pool 1 (Token1/Token2)**: Check deployment logs for the assigned ID
- **Pool 2 (Token2/Token3)**: Check deployment logs for the assigned ID

The factory provides methods for:
- Creating new pools
- Finding existing pools  
- Performing multi-hop swaps
- Querying pool information

## Security Considerations

- The script uses a default passphrase for regtest environments
- For production deployments, use strong passphrases and secure wallet storage
- Always verify contract addresses before interacting with them
- Test thoroughly on regtest before deploying to mainnet

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review the deezel documentation in `./reference/deezel/README.md`
3. Examine the test suite in `src/tests/` for reference implementations
4. Check the Alkanes documentation for cellpack notation details