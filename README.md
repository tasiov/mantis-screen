# Mantis Raydium Client

A command-line tool for interacting with Raydium pools on Solana. Note that this tool only works with Raydium v4 pools.

## Prerequisites

- Rust and Cargo (v1.83.0)
- Solana CLI tools (v1.18.17)
- A valid Solana keypair file

## Quick Start

1. Clone the repository:

```bash
git clone [repository-url]
cd mantis-raydium-client
```

2. Create a config file at `./config.toml`:

```toml
rpc_url = "https://api.mainnet-beta.solana.com"  # or your preferred RPC
api_key = "YOUR_API_KEY"
keypair_path = "./keypair.json"        # path to your keypair
```

3. Build and run:

```bash
# Build the project
cargo build --release

# Run the binary
./target/release/mantis-raydium-client --help
```

## Example Commands

Add liquidity to a pool:

```bash
./target/release/mantis-raydium-client add-liquidity \
    --pool-id AgFnRLUScRD2E4nWQxW73hdbSN7eKEUb2jHX7tx9YTYc \
    --lp-amount 0.000288889 \
    --slippage-percentage 1 \
    --base-amount-min 0.01105525 \
    --quote-amount-min 0.000010006
```

Remove liquidity from a pool:

```bash
./target/release/mantis-raydium-client remove-liquidity \
    --pool-id AgFnRLUScRD2E4nWQxW73hdbSN7eKEUb2jHX7tx9YTYc \
    --lp-amount 0.000288889 \
    --slippage-percentage 1 \
    --base-amount-min 0.01105525 \
    --quote-amount-min 0.000010006
```

## Available Commands

- `fetch-pool-info`: Fetch pool data by pool id
- `fetch-pool-keys`: Fetch pool keys by pool id
- `add-liquidity`: Add liquidity to a Raydium pool
- `remove-liquidity`: Remove liquidity from a Raydium pool

Use `--help` with any command to see detailed usage information:

```bash
./target/release/mantis-raydium-client add-liquidity --help
```
