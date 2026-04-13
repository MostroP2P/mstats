# Implementation Plan: Mostro Stats CLI

**Branch**: `001-mostro-stats-cli` | **Date**: 2026-04-13 | **Spec**: [specs/001-mostro-stats-cli/spec.md](spec.md)
**Input**: Feature specification from `/specs/001-mostro-stats-cli/spec.md`

## Summary

Build `mstats`, a Rust CLI tool that analyzes Mostro P2P trading activity by fetching kind 8383 development fee events and kind 38383 order events from a single Nostr relay (`wss://relay.mostro.network`), joining them by order ID, and producing global and per-node aggregated statistics. Kind 38383 fetching is mandatory. Order IDs are deduplicated and kind 38383 events are fetched in a single batched relay query. The tool supports filtering by date range, node pubkey, fiat currency (case-insensitive, normalized to uppercase), and order side (buy/sell), with both human-readable default output (including data quality summary) and optional JSON output (including event-level trace IDs).

## Technical Context

**Language/Version**: Rust 2021 edition (latest stable toolchain available at build time)
**Primary Dependencies**: `nostr-sdk` 0.44.1 (constitution-mandated), `clap` (CLI argument parsing), `serde` / `serde_json` (JSON serialization)
**Storage**: N/A — no persistent storage in v1; all data processed in-memory from live relay queries
**Testing**: `cargo test` with unit tests; optional integration tests that query the live relay (gated behind a feature flag to keep CI fast and offline-compatible)
**Target Platform**: Linux x86_64 (primary), macOS aarch64/x86_64 (secondary), Windows x86_64 (best-effort)
**Project Type**: CLI tool
**Performance Goals**: Complete full report within 60 seconds for up to 10,000 kind 8383 events on the relay (SC-001)
**Constraints**: Single relay only (v1); no web UI, no charts, no database, no multi-relay, no real-time monitoring (constitution Principle VII); `nostr-sdk` 0.44.1 is the only permitted Nostr library (constitution Principle VI)
**Scale/Scope**: Up to ~10,000 kind 8383 events; in-memory aggregation; no pagination beyond relay-native limits

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Gate | Status |
|-----------|------|--------|
| I. Correctness Over Cleverness | All statistics must be directly derived from Nostr events; no inferred metrics | ✅ PASS — Design uses only real relay events; join-based aggregation ensures traceability |
| II. Single Canonical Data Source (v1) | Must use only `wss://relay.mostro.network` | ✅ PASS — Relay is hardcoded; no relay discovery or multi-relay logic |
| III. Small and Maintainable CLI-First Design | Must remain dependency-light; clear Rust CLI architecture | ✅ PASS — Minimal dependencies (nostr-sdk, clap, serde); no frameworks, no DB, no UI |
| IV. Transparent Handling of Incomplete Data | Missing tags, unjoined events, relay failures must be explicit | ✅ PASS — Design includes unjoined-event reporting and error-path visibility |
| V. Human and Machine Readable Output | Default human-readable + optional JSON | ✅ PASS — Two output modes: tabular text (default) and JSON (flag-gated) |
| VI. Canonical Nostr Implementation | Must use `nostr-sdk` 0.44.1 | ✅ PASS — Pinned in Cargo.toml; no alternative Nostr crates |
| VII. Explicit Scope for v1 | No web UI, charts, DB, scoring, multi-relay, real-time | ✅ PASS — None of these are in scope; spec explicitly excludes them |

**All gates passed. Phase 0 research complete. Post-Phase 1 re-evaluation:**

| Principle | Gate | Status |
|-----------|------|--------|
| I. Correctness Over Cleverness | All statistics derived from real events, traceable to event IDs | ✅ PASS — data-model.md confirms join-based aggregation; unjoined events excluded from aggregates |
| II. Single Canonical Data Source (v1) | Relay hardcoded to `wss://relay.mostro.network` | ✅ PASS — contracts show relay URL hardcoded in `config.rs` with no user-overridable flag; no multi-relay logic. Relay override is explicitly out-of-scope for v1. |
| III. Small and Maintainable CLI-First Design | Flat module layout, minimal deps, no frameworks | ✅ PASS — 9 source modules, no DB, no UI; flat structure aids auditability |
| IV. Transparent Handling of Incomplete Data | Unjoined events and errors reported explicitly | ✅ PASS — `UnjoinedRecord` and `errors` array in output contract; error paths to stderr |
| V. Human and Machine Readable Output | Two output modes: tabular + JSON | ✅ PASS — Both output contracts defined with consistent data shape |
| VI. Canonical Nostr Implementation | `nostr-sdk` 0.44.1 pinned | ✅ PASS — Listed in Technical Context and Cargo.toml requirement |
| VII. Explicit Scope for v1 | No web UI, charts, DB, scoring, multi-relay, real-time | ✅ PASS — None present in source structure, data model, or contracts |

**All gates remain PASS after Phase 1 design. Plan is ready for `/speckit.tasks`.**

## Project Structure

### Documentation (this feature)

```text
specs/001-mostro-stats-cli/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point, argument parsing, output dispatch
├── cli.rs               # clap CLI definition and subcommands
├── config.rs            # Configuration (relay URL, defaults)
├── relay.rs             # Nostr relay client wrapper (nostr-sdk 0.44.1)
├── event_parser.rs      # Parsing kind 8383 and kind 38383 events
├── joiner.rs            # Join logic: match order-id to d tag
├── aggregator.rs        # Global and per-node statistics computation
├── models.rs            # Core data structures (events, joined records, stats)
├── filters.rs           # Date, node, currency, side filter logic
└── output.rs            # Human-readable and JSON output formatters

tests/
├── unit/
│   ├── test_event_parser.rs
│   ├── test_joiner.rs
│   ├── test_aggregator.rs
│   └── test_filters.rs
└── integration/
    └── test_relay_query.rs   # Requires live relay connection (feature-gated)
```

**Structure Decision**: Single-project Rust CLI (Option 1). The `src/` tree at repository root with a flat module layout keeps the codebase auditable and small, aligning with Constitution Principle III. Tests are split into unit (offline, always run) and integration (relay-dependent, feature-gated).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations found. All constitution principles are satisfied by the design above.
