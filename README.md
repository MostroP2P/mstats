# mstats

Mostro trading statistics CLI. Fetches kind 8383 development fee events and kind 38383 order events from `wss://relay.mostro.network`, joins them by order ID, and produces global and per-node aggregated statistics.

## Quick Start

```bash
# Build
cargo build --release

# Run (human-readable report)
./target/release/mstats

# JSON output
./target/release/mstats --json

# Filter by date range
./target/release/mstats --from 2026-01-01 --to 2026-03-31

# Filter by node, currency, and side
./target/release/mstats --node <pubkey> --currency USD --side buy
```

## Features

- **Global and per-node statistics**: order count, total dev fees, total volume, average order size, fiat volume by currency, volume by side
- **Filtering**: by date range, node pubkey, fiat currency, and order side (buy/sell)
- **Output formats**: human-readable (default) and JSON (`--json`)
- **Data quality summary**: every run reports processed/joined/unmatched/skipped counts
- **Event traceability**: JSON output includes source event IDs for audit

## Constraints (v1)

- Single relay: `wss://relay.mostro.network`
- No persistent database, web UI, charts, or multi-relay support
- Uses `nostr-sdk` 0.44.1

## Development

```bash
cargo fmt && cargo clippy && cargo test
```

Integration tests (require relay access):
```bash
cargo test --features integration-tests
```
