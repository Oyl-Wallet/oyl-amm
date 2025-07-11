#!/bin/bash

# OYL Protocol AMM Deployment Script
# This script deploys the complete OYL AMM system using the deezel CLI tool
# Usage: ./deploy-oyl-amm.sh -p regtest --sandshrew-rpc-url http://localhost:18888 [options]

set -e

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALLET_FILE="${HOME}/.deezel/regtest.json.asc"
PASSPHRASE="testtesttest"
FEE_RATE=1
MINE_BLOCKS=true
AUTO_CONFIRM=true
USE_OYL_SDK=false

# Default values
PROVIDER="regtest"
SANDSHREW_RPC_URL=""
BITCOIN_RPC_URL=""
DEEZEL_BINARY="deezel"
OYL_PROTOCOL_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OYL_BINARY="oyl"
ALKANES_DIRECTORY=""

# Contract deployment constants (matching test suite)
AMM_FACTORY_ID=65522
AUTH_TOKEN_FACTORY_ID=65517
AMM_FACTORY_PROXY_TX=1
AMM_FACTORY_LOGIC_IMPL_TX=$((0xf3ff))
POOL_BEACON_PROXY_TX=$((0xbeac1))
POOL_UPGRADEABLE_BEACON_TX=$((0xbeac0))
OWNED_TOKEN_1_DEPLOYMENT_TX=3
OWNED_TOKEN_2_DEPLOYMENT_TX=5
OWNED_TOKEN_3_DEPLOYMENT_TX=7
OYL_TOKEN_DEPLOYMENT_TX=9
EXAMPLE_FLASHSWAP_TX=10

# Token initialization amounts
INIT_AMT_TOKEN1=1000000000000000000000
INIT_AMT_TOKEN2=2000000000000000000000
INIT_AMT_TOKEN3=1000000000000000000000
INIT_AMT_OYL=1000000000000000000000

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
OYL Protocol AMM Deployment Script

USAGE:
    $0 -p PROVIDER --sandshrew-rpc-url URL [OPTIONS]

REQUIRED ARGUMENTS:
    -p, --provider PROVIDER              Network provider (regtest, testnet, signet, mainnet)
    --sandshrew-rpc-url URL             Sandshrew RPC endpoint URL

OPTIONAL ARGUMENTS:
    --bitcoin-rpc-url URL               Bitcoin RPC endpoint URL
    --deezel-binary PATH                Path to deezel binary (default: deezel)
    --oyl-binary PATH                   Path to oyl binary (default: oyl)
    --use-oyl-sdk                       Use the oyl-sdk CLI instead of deezel
    --alkanes-directory PATH            Path to alkanes-rs directory for building standard contracts
    --wallet-file PATH                  Path to wallet file (default: ~/.deezel/regtest.json.asc)
    --passphrase PASS                   Wallet passphrase (default: testtesttest)
    --fee-rate RATE                     Transaction fee rate in sat/vB (default: 1)
    --no-mine                           Don't mine blocks after transactions
    -y, --yes                           Auto-confirm all transactions
    -h, --help                          Show this help message

EXAMPLES:
    # Deploy to local regtest
    $0 -p regtest --sandshrew-rpc-url http://localhost:18888

    # Deploy with custom deezel binary
    $0 -p regtest --sandshrew-rpc-url http://localhost:18888 \\
       --deezel-binary /data/deezel-old/target/release/deezel

    # Deploy with custom alkanes directory
    $0 -p regtest --sandshrew-rpc-url http://localhost:18888 \\
       --alkanes-directory /home/ubuntu/alkanes-rs

    # Deploy to testnet with custom settings
    $0 -p testnet --sandshrew-rpc-url https://testnet.sandshrew.io \\
       --wallet-file ~/.deezel/testnet.json.asc --fee-rate 5

    # Deploy with auto-confirmation
    $0 -p regtest --sandshrew-rpc-url http://localhost:18888 -y

DEPLOYMENT SEQUENCE:
    1. Setup wallet and fund with Bitcoin
    2. Deploy AMM pool logic implementation
    3. Deploy auth token factory
    4. Deploy AMM factory logic implementation
    5. Deploy test tokens (Token1, Token2, Token3, OYL)
    6. Deploy example flashswap contract
    7. Deploy beacon proxy and upgradeable beacon
    8. Deploy AMM factory proxy and initialize
    9. Create initial liquidity pools
    10. Verify deployment

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--provider)
                PROVIDER="$2"
                shift 2
                ;;
            --sandshrew-rpc-url)
                SANDSHREW_RPC_URL="$2"
                shift 2
                ;;
            --bitcoin-rpc-url)
                BITCOIN_RPC_URL="$2"
                shift 2
                ;;
            --deezel-binary)
                DEEZEL_BINARY="$2"
                shift 2
                ;;
            --oyl-binary)
                OYL_BINARY="$2"
                shift 2
                ;;
            --use-oyl-sdk)
                USE_OYL_SDK=true
                shift
                ;;
            --alkanes-directory)
                ALKANES_DIRECTORY="$2"
                shift 2
                ;;
            --wallet-file)
                WALLET_FILE="$2"
                shift 2
                ;;
            --passphrase)
                PASSPHRASE="$2"
                shift 2
                ;;
            --fee-rate)
                FEE_RATE="$2"
                shift 2
                ;;
            --no-mine)
                MINE_BLOCKS=false
                shift
                ;;
            -y|--yes)
                AUTO_CONFIRM=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Validate required arguments
    if [[ -z "$SANDSHREW_RPC_URL" ]]; then
        log_error "Missing required argument: --sandshrew-rpc-url"
        show_help
        exit 1
    fi
}

# Build deezel command with common options
build_deezel_cmd() {
    local cmd="$DEEZEL_BINARY"
    
    if [[ -n "$PROVIDER" ]]; then
        cmd="$cmd -p $PROVIDER"
    fi
    
    if [[ -n "$SANDSHREW_RPC_URL" ]]; then
        cmd="$cmd --sandshrew-rpc-url $SANDSHREW_RPC_URL"
    fi
    
    if [[ -n "$BITCOIN_RPC_URL" ]]; then
        cmd="$cmd --bitcoin-rpc-url $BITCOIN_RPC_URL"
    fi
    
    if [[ -n "$WALLET_FILE" ]]; then
        cmd="$cmd --wallet-file $WALLET_FILE"
    fi
    
    if [[ -n "$PASSPHRASE" ]]; then
        cmd="$cmd --passphrase $PASSPHRASE"
    fi
    
    echo "$cmd"
}

build_oyl_cmd() {
    local sub_command="$1"
    local cmd="$OYL_BINARY $sub_command"

    # Add flags based on the subcommand
    case $sub_command in
        "alkane")
            if [[ -n "$PROVIDER" ]]; then
                cmd="$cmd --provider $PROVIDER"
            fi
            if [[ -n "$SANDSHREW_RPC_URL" ]]; then
                cmd="$cmd --metashrew-rpc-url $SANDSHREW_RPC_URL"
            fi
            ;;
        "regtest"|"utxo")
            # These commands are called with explicit flags in their respective functions
            ;;
    esac
    
    echo "$cmd"
}

mine_blocks() {
    log_info "Mining 1 block..."
    local oyl_cmd="$OYL_BINARY"
    $oyl_cmd regtest genBlocks --blocks 1 --provider $PROVIDER
    sleep 3 # Give time for sync
}

# Check if dependencies are available
check_dependencies() {
    if [[ "$USE_OYL_SDK" == "true" ]]; then
        if ! command -v "jq" &> /dev/null; then
            log_error "jq is not installed. Please install jq to use the oyl-sdk backend."
            exit 1
        fi
        log_info "Found jq: $(which jq)"

        # If OYL_BINARY is not a full path, try to find it
        if ! [[ "$OYL_BINARY" == /* ]]; then
            if command -v "$OYL_BINARY" &> /dev/null; then
                OYL_BINARY=$(command -v "$OYL_BINARY")
                log_info "Found oyl in PATH: $OYL_BINARY"
            else
                local local_path="${OYL_PROTOCOL_DIRECTORY}/reference/oyl-sdk/bin/oyl.js"
                if [[ -f "$local_path" ]]; then
                    OYL_BINARY="$local_path"
                    log_info "Found oyl in reference directory: $OYL_BINARY"
                else
                    log_error "oyl binary not found: $OYL_BINARY"
                    log_error "Please ensure oyl-sdk is installed, in your PATH, or specify the path with --oyl-binary."
                    exit 1
                fi
            fi
        fi

        if [[ ! -f "$OYL_BINARY" ]]; then
            log_error "oyl binary not found at path: $OYL_BINARY"
            exit 1
        fi

        if [[ ! -x "$OYL_BINARY" ]]; then
            log_warning "oyl binary is not executable. Attempting to set permissions..."
            chmod +x "$OYL_BINARY"
            if [[ ! -x "$OYL_BINARY" ]]; then
                log_error "Failed to make oyl binary executable. Please check permissions."
                exit 1
            fi
        fi
    else
        if ! command -v "$DEEZEL_BINARY" &> /dev/null; then
            log_error "deezel binary not found: $DEEZEL_BINARY"
            log_error "Please ensure deezel is installed or use --deezel-binary to specify the path."
            exit 1
        fi
        log_info "Found deezel: $(which "$DEEZEL_BINARY")"
    fi
}

# Setup wallet and fund it
setup_wallet() {
    log_info "Setting up wallet..."

    if [[ "$USE_OYL_SDK" == "true" ]]; then
        local oyl_cmd="$OYL_BINARY"
        local faucet_address="bcrt1qzr9vhs60g6qlmk7x3dd7g3ja30wyts48sxuemv"
        local test_wallet_address="bcrt1qcr8te4kr609gcawutmrza0j4xv80jy8zeqchgx"

        log_info "Ensuring regtest faucet is funded..."
        $oyl_cmd regtest genBlocks --count 101 --address "$faucet_address" --provider $PROVIDER
        
        log_info "Maturing faucet funds..."
        $oyl_cmd regtest genBlocks --count 100 --provider $PROVIDER

        log_info "Waiting for faucet UTXOs to be indexed..."
        local attempts=0
        local max_attempts=20
        local utxos_json=""
        while [[ $attempts -lt $max_attempts ]]; do
            utxos_json=$($oyl_cmd utxo addressUtxos --address "$faucet_address" --provider "$PROVIDER" 2>/dev/null)
            if [[ -n "$utxos_json" && "$utxos_json" != "[]" && $(echo "$utxos_json" | jq 'length') -gt 0 ]]; then
                log_info "Faucet has spendable UTXOs."
                break
            fi
            log_warning "Faucet has no spendable UTXOs yet. Waiting... (Attempt $((attempts+1))/$max_attempts)"
            sleep 3
            attempts=$((attempts+1))
        done

        if [[ -z "$utxos_json" || "$utxos_json" == "[]" || $(echo "$utxos_json" | jq 'length') -eq 0 ]]; then
            log_error "Faucet funding failed. No UTXOs found after $max_attempts attempts."
            exit 1
        fi

        log_info "Funding test wallet from faucet..."
        $oyl_cmd regtest sendFromFaucet --to "$test_wallet_address" --provider $PROVIDER
        
        mine_blocks # Mine one more block to confirm the funding transaction
        
        log_info "Checking wallet balance..."
        $oyl_cmd utxo balance --provider $PROVIDER
    else
        local deezel_cmd=$(build_deezel_cmd)
        
        # Remove existing wallet to ensure clean state
        if [[ -f "$WALLET_FILE" ]]; then
            log_warning "Removing existing wallet file: $WALLET_FILE"
            rm -f "$WALLET_FILE"
        fi
        
        # Create new wallet
        log_info "Creating new GPG-encrypted wallet..."
        $deezel_cmd wallet create
        
        # Check initial UTXOs
        log_info "Checking initial UTXOs..."
        $deezel_cmd wallet utxos --addresses p2tr:0 || true
        
        # Generate blocks to fund wallet
        log_info "Generating 201 blocks to P2TR address for funding..."
        $deezel_cmd bitcoind generatetoaddress 201 [self:p2tr:0]
        
        # Wait for blockchain sync
        log_info "Waiting for blockchain sync..."
        sleep 6
        
        # Check UTXOs after funding
        log_info "Checking UTXOs after block generation..."
        $deezel_cmd wallet utxos --addresses p2tr:0
    fi
    
    log_success "Wallet setup complete"
}

# Deploy a contract using envelope pattern
deploy_contract() {
    local contract_name="$1"
    local contract_base_name="$2"
    local cellpack_inputs="$3"
    local target_tx="$4"
    
    log_info "Deploying $contract_name..."
    
    # Find the contract file in the appropriate directories
    local contract_file=""
    
    # Check OYL Protocol contracts first
    local oyl_target="${OYL_PROTOCOL_DIRECTORY}/target/wasm32-unknown-unknown/release"
    local oyl_deps="${oyl_target}/deps"
    
    if [[ -f "${oyl_target}/${contract_base_name}.wasm" ]]; then
        contract_file="${oyl_target}/${contract_base_name}.wasm"
    elif [[ -f "${oyl_deps}/${contract_base_name}.wasm" ]]; then
        contract_file="${oyl_deps}/${contract_base_name}.wasm"
    fi
    
    # If not found in OYL Protocol, check Alkanes directory
    if [[ -z "$contract_file" && -n "$ALKANES_DIRECTORY" && -d "$ALKANES_DIRECTORY" ]]; then
        local alkanes_target="${ALKANES_DIRECTORY}/target/wasm32-unknown-unknown/release"
        local alkanes_deps="${alkanes_target}/deps"
        
        if [[ -f "${alkanes_target}/${contract_base_name}.wasm" ]]; then
            contract_file="${alkanes_target}/${contract_base_name}.wasm"
        elif [[ -f "${alkanes_deps}/${contract_base_name}.wasm" ]]; then
            contract_file="${alkanes_deps}/${contract_base_name}.wasm"
        fi
    fi
    
    if [[ -z "$contract_file" ]]; then
        log_error "Contract file not found: ${contract_base_name}.wasm"
        log_error "Searched in:"
        log_error "  - OYL Protocol: ${oyl_target}"
        log_error "  - OYL Protocol deps: ${oyl_deps}"
        if [[ -n "$ALKANES_DIRECTORY" ]]; then
            log_error "  - Alkanes: ${alkanes_target}"
            log_error "  - Alkanes deps: ${alkanes_deps}"
        fi
        exit 1
    fi
    
    log_info "Using contract file: $contract_file"
    
    if [[ "$USE_OYL_SDK" == "true" ]]; then
        local oyl_cmd=$(build_oyl_cmd "alkane")
        local calldata="3,${target_tx},${cellpack_inputs}"
        
        $oyl_cmd alkane new-contract --contract "$contract_file" --calldata "$calldata" --feeRate $FEE_RATE
        
        if [[ "$MINE_BLOCKS" == "true" ]]; then
            mine_blocks
        fi
    else
        local deezel_cmd=$(build_deezel_cmd)
        local confirm_flag=""
        if [[ "$AUTO_CONFIRM" == "true" ]]; then
            confirm_flag="-y"
        fi
        
        local mine_flag=""
        if [[ "$MINE_BLOCKS" == "true" ]]; then
            mine_flag="--mine"
        fi
        
        # Build the cellpack notation: [3, target_tx, inputs...]
        local cellpack="[3,$target_tx,$cellpack_inputs]"
        
        $deezel_cmd alkanes execute \
            --inputs B:10000 \
            --change [self:p2tr:2] \
            --to [self:p2tr:1] \
            --envelope "$contract_file" \
            $mine_flag \
            --fee-rate $FEE_RATE \
    	--trace \
            $confirm_flag \
            "$cellpack:v0:v0"
    fi
    
    log_success "$contract_name deployed successfully"
}

# Deploy contracts without envelope (for simple cellpack operations)
deploy_cellpack() {
    local operation_name="$1"
    local cellpack_notation="$2"
    
    log_info "Executing $operation_name..."
    
    if [[ "$USE_OYL_SDK" == "true" ]]; then
        local oyl_cmd=$(build_oyl_cmd "alkane")
        # Extract calldata from cellpack notation (e.g., "[3,1,0,65521,4,65520]")
        local calldata=$(echo "$cellpack_notation" | sed 's/\[//g' | sed 's/\]//g')
        
        $oyl_cmd alkane execute --calldata "$calldata" --feeRate $FEE_RATE
        
        if [[ "$MINE_BLOCKS" == "true" ]]; then
            mine_blocks
        fi
    else
        local deezel_cmd=$(build_deezel_cmd)
        local confirm_flag=""
        if [[ "$AUTO_CONFIRM" == "true" ]]; then
            confirm_flag="-y"
        fi
        
        local mine_flag=""
        if [[ "$MINE_BLOCKS" == "true" ]]; then
            mine_flag="--mine"
        fi
        
        $deezel_cmd alkanes execute \
            --inputs B:10000 \
            --change [self:p2tr:2] \
            --to [self:p2tr:1] \
            $mine_flag \
            --fee-rate $FEE_RATE \
    	--trace \
            $confirm_flag \
            "$cellpack_notation:v0:v0"
    fi
    
    log_success "$operation_name completed successfully"
}

# Build contracts if needed
build_contracts() {
    log_info "Building contracts..."
    
    # Build OYL Protocol contracts
    log_info "Building OYL Protocol contracts in: $OYL_PROTOCOL_DIRECTORY"
    cd "$OYL_PROTOCOL_DIRECTORY"
    
    log_info "Building pool contract..."
    cargo build --release -p pool
    
    log_info "Building factory contract..."
    cargo build --release -p factory
    
    log_info "Building all OYL Protocol contracts..."
    cargo build --release
    
    # Build Alkanes standard contracts if directory is provided
    if [[ -n "$ALKANES_DIRECTORY" && -d "$ALKANES_DIRECTORY" ]]; then
        log_info "Building Alkanes standard contracts in: $ALKANES_DIRECTORY"
        cd "$ALKANES_DIRECTORY"
        
        log_info "Building alkanes-std-auth-token..."
        cargo build -p alkanes-std-auth-token --release
        
        log_info "Building alkanes-std-owned-token..."
        cargo build -p alkanes-std-owned-token --release
        
        log_info "Building alkanes-std-beacon-proxy..."
        cargo build -p alkanes-std-beacon-proxy --release
        
        log_info "Building alkanes-std-upgradeable-beacon..."
        cargo build -p alkanes-std-upgradeable-beacon --release
        
        log_info "Building alkanes-std-upgradeable..."
        cargo build -p alkanes-std-upgradeable --release
    else
        log_warning "Alkanes directory not provided or doesn't exist. Standard contracts may not be available."
        log_info "Use --alkanes-directory to specify the path to alkanes-rs"
    fi
    
    # Return to original directory
    cd "$OYL_PROTOCOL_DIRECTORY"
    
    log_success "Contract build complete"
}

# Deploy the complete AMM system
deploy_amm_system() {
    log_info "Starting OYL AMM system deployment..."
    
    # Phase 1: Deploy core contracts with envelope pattern
    log_info "Phase 1: Deploying core contracts..."
    
    # Deploy AMM pool logic implementation (target: [3, AMM_FACTORY_ID])
    deploy_contract "AMM Pool Logic" \
        "pool" \
        "50" \
        "$AMM_FACTORY_ID"
    
    # Deploy auth token factory (target: [3, AUTH_TOKEN_FACTORY_ID])
    deploy_contract "Auth Token Factory" \
        "alkanes_std_auth_token" \
        "100" \
        "$AUTH_TOKEN_FACTORY_ID"
    
    # Deploy AMM factory logic implementation
    deploy_contract "AMM Factory Logic" \
        "factory" \
        "50" \
        "$AMM_FACTORY_LOGIC_IMPL_TX"
    
    # Phase 2: Deploy tokens
    log_info "Phase 2: Deploying tokens..."
    
    # Deploy Token 1 (owned token with initial mint)
    deploy_contract "Token 1" \
        "alkanes_std_owned_token" \
        "0,1,$INIT_AMT_TOKEN1" \
        "$OWNED_TOKEN_1_DEPLOYMENT_TX"
    
    # Deploy Token 2
    deploy_contract "Token 2" \
        "alkanes_std_owned_token" \
        "0,1,$INIT_AMT_TOKEN2" \
        "$OWNED_TOKEN_2_DEPLOYMENT_TX"
    
    # Deploy Token 3
    deploy_contract "Token 3" \
        "alkanes_std_owned_token" \
        "0,1,$INIT_AMT_TOKEN3" \
        "$OWNED_TOKEN_3_DEPLOYMENT_TX"
    
    # Deploy OYL Token with name and symbol
    local oyl_name_hex=$(echo -n "OYL Token" | xxd -p | tr -d '\n')
    local oyl_symbol_hex=$(echo -n "OYL" | xxd -p | tr -d '\n')
    # Pad to 16 bytes (32 hex chars)
    oyl_name_hex="${oyl_name_hex}$(printf '%*s' $((32 - ${#oyl_name_hex})) '' | tr ' ' '0')"
    oyl_symbol_hex="${oyl_symbol_hex}$(printf '%*s' $((32 - ${#oyl_symbol_hex})) '' | tr ' ' '0')"
    
    deploy_contract "OYL Token" \
        "oyl_token" \
        "0,$INIT_AMT_OYL,0x$oyl_name_hex,0x$oyl_symbol_hex" \
        "$OYL_TOKEN_DEPLOYMENT_TX"
    
    # Phase 3: Deploy infrastructure contracts
    log_info "Phase 3: Deploying infrastructure contracts..."
    
    # Deploy example flashswap
    deploy_contract "Example Flashswap" \
        "example_flashswap" \
        "0" \
        "$EXAMPLE_FLASHSWAP_TX"
    
    # Deploy beacon proxy
    deploy_contract "Beacon Proxy" \
        "alkanes_std_beacon_proxy" \
        "$((0x8fff))" \
        "$POOL_BEACON_PROXY_TX"
    
    # Deploy upgradeable beacon
    deploy_contract "Upgradeable Beacon" \
        "alkanes_std_upgradeable_beacon" \
        "$((0x7fff)),4,$AMM_FACTORY_ID,1" \
        "$POOL_UPGRADEABLE_BEACON_TX"
    
    # Phase 4: Deploy and initialize factory proxy
    log_info "Phase 4: Deploying factory proxy..."
    
    # Deploy factory proxy
    deploy_contract "Factory Proxy" \
        "alkanes_std_upgradeable" \
        "$((0x7fff)),4,$AMM_FACTORY_LOGIC_IMPL_TX,1" \
        "$AMM_FACTORY_PROXY_TX"
    
    # Initialize factory proxy
    deploy_cellpack "Factory Initialization" \
        "[3,$AMM_FACTORY_PROXY_TX,0,$POOL_BEACON_PROXY_TX,4,$POOL_UPGRADEABLE_BEACON_TX]"
    
    log_success "OYL AMM system deployment complete!"
}

# Create initial liquidity pools
create_initial_pools() {
    log_info "Creating initial liquidity pools..."
    
    # Create pool 1: Token1/Token2
    log_info "Creating Token1/Token2 pool..."
    deploy_cellpack "Pool 1 Creation" \
        "[3,$AMM_FACTORY_PROXY_TX,1,4,$OWNED_TOKEN_1_DEPLOYMENT_TX,4,$OWNED_TOKEN_2_DEPLOYMENT_TX,1000000,1000000]"
    
    # Create pool 2: Token2/Token3
    log_info "Creating Token2/Token3 pool..."
    deploy_cellpack "Pool 2 Creation" \
        "[3,$AMM_FACTORY_PROXY_TX,1,4,$OWNED_TOKEN_2_DEPLOYMENT_TX,4,$OWNED_TOKEN_3_DEPLOYMENT_TX,1000000,1000000]"
    
    log_success "Initial liquidity pools created!"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    if [[ "$USE_OYL_SDK" == "true" ]]; then
        local oyl_cmd="$OYL_BINARY"
        log_info "Checking wallet balance..."
        $oyl_cmd utxo balance --provider $PROVIDER
        
        local factory_cmd=$(build_oyl_cmd "alkane")
        log_info "Checking factory status..."
        $factory_cmd alkane get-all-pools-details --target "4:$AMM_FACTORY_PROXY_TX"
    else
        local deezel_cmd=$(build_deezel_cmd)
        
        # Check wallet balance
        log_info "Checking wallet balance..."
        $deezel_cmd wallet balance --addresses p2tr:0-5
        
        # Check alkanes balances
        log_info "Checking alkanes token balances..."
        $deezel_cmd alkanes balance --address [self:p2tr:0] || true
        
        # Get factory info (number of pools)
        log_info "Checking factory status..."
        deploy_cellpack "Get Pool Count" "[3,$AMM_FACTORY_PROXY_TX,4]"
    fi
    
    log_success "Deployment verification complete!"
}

# Main execution
main() {
    log_info "OYL Protocol AMM Deployment Script"
    log_info "=================================="
    
    parse_args "$@"
    
    log_info "Configuration:"
    log_info "  Provider: $PROVIDER"
    log_info "  Sandshrew RPC: $SANDSHREW_RPC_URL"
    log_info "  Bitcoin RPC: ${BITCOIN_RPC_URL:-"(default)"}"
    if [[ "$USE_OYL_SDK" == "true" ]]; then
        log_info "  Backend: oyl-sdk"
        log_info "  OYL Binary: $OYL_BINARY"
    else
        log_info "  Backend: deezel"
        log_info "  Deezel Binary: $DEEZEL_BINARY"
    fi
    log_info "  OYL Protocol Directory: $OYL_PROTOCOL_DIRECTORY"
    log_info "  Alkanes Directory: ${ALKANES_DIRECTORY:-"(not specified)"}"
    log_info "  Wallet File: $WALLET_FILE"
    log_info "  Fee Rate: $FEE_RATE sat/vB"
    log_info "  Mine Blocks: $MINE_BLOCKS"
    log_info "  Auto Confirm: $AUTO_CONFIRM"
    
    check_dependencies
    build_contracts
    setup_wallet
    deploy_amm_system
    create_initial_pools
    verify_deployment
    
    log_success "🎉 OYL Protocol AMM deployment completed successfully!"
    log_info "The AMM is now ready for use on $PROVIDER network"
    log_info "Factory Proxy ID: 4:$AMM_FACTORY_PROXY_TX"
    log_info "Pool 1 (Token1/Token2): Check deployment logs for ID"
    log_info "Pool 2 (Token2/Token3): Check deployment logs for ID"
}

# Run main function with all arguments
main "$@"

