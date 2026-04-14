# Research: Mostro Stats CLI

**Date**: 2026-04-13
**Feature**: 001-mostro-stats-cli

## Decision: Nostr SDK selection and version pinning

**Chosen**: `nostr-sdk` crate version `0.44.1`

**Rationale**: This is mandated by the constitution (Principle VI — Canonical Nostr Implementation). The `nostr-sdk` crate provides relay connection management, event querying, subscription handling, and event parsing. Version 0.44.1 is pinned exactly in `Cargo.toml` to ensure reproducible builds.

**Alternatives considered**:
- `nostr` crate (lower-level) — rejected because constitution requires nostr-sdk
- Custom WebSocket implementation — rejected: violates principle VI, increases attack surface, unnecessary reinvention

## Decision: Relay connection strategy

**Chosen**: Connect to a single relay (`wss://relay.mostro.network`) with a timeout-based connection model. The tool establishes a single WebSocket connection, subscribes to kind 8383 events (and optionally kind 38383 events if supported by subscription filters), collects events, then disconnects.

**Rationale**: Constitution Principle II mandates a single canonical relay. A single connection simplifies the architecture and eliminates multi-relay deduplication complexity.

**Alternatives considered**:
- Batch relay connections — rejected: v1 scope forbids multi-relay
- Persistent connection — rejected: v1 is a run-to-completion CLI, not a daemon

## Decision: Event fetching pattern

**Chosen**: Two-phase fetching with **batched** kind 38383 queries:
1. Phase A: Query all kind 8383 events from the relay, extract order IDs and node pubkeys
2. Phase B: Deduplicate order IDs (unique set) → single batched relay query for kind 38383 events filtering on `d` tags matching all unique order IDs

Kind 38383 fetching is **mandatory** — statistics cannot be produced without it.

**Rationale**: Kind 8383 events are the entry point (constitution Principle VII). Deduplicating order IDs before the kind 38383 query eliminates N+1 roundtrips. nostr-sdk supports filtering on multiple `d` tag values in a single subscription, making this a single relay query regardless of how many unique order IDs exist (subject to relay-native limits on filter size).

**Alternatives considered**:
- Individual 38383 lookups per order ID — rejected: N+1 query pattern, unacceptably slow for large datasets
- Single subscription for both kinds simultaneously — viable but adds complexity; two-phase approach is simpler and gives us the complete order ID set before querying 38383s

## Decision: CLI framework

**Chosen**: `clap` (derive mode) for argument parsing

**Rationale**: `clap` is the de facto standard Rust CLI framework. The derive mode provides type-safe argument definitions with minimal boilerplate. It supports subcommands, which aligns with the filtering requirements (date, node, currency, side).

**Alternatives considered**:
- `structopt` — deprecated; merged into clap derive mode
- Manual argument parsing — rejected: unnecessary complexity for a well-solved problem

## Decision: Event timestamp for date filtering

**Chosen**: Use the `created_at` field from the kind 8383 event as the primary filter for date range queries.

**Rationale**: The kind 8383 event represents the development fee payment, which is the entry point for v1 statistics. Its `created_at` timestamp is when the fee was paid. This is a single, unambiguous rule: **date filtering applies to the kind 8383 event's `created_at` timestamp**. If the kind 8383 falls outside the date range, the entire joined record is excluded regardless of the kind 38383 event's timestamp.

**Alternatives considered**:
- Use kind 38383 `created_at` — rejected: order events may have been created before or after the fee payment; the fee event timestamp is the authoritative anchor
- Use both timestamps — rejected: introduces ambiguity; v1 needs one clear rule

## Decision: Output format structure

**Chosen**: Two output modes controlled by a `--json` flag:
- **Human-readable (default)**: Tabular text output to stdout with global statistics first, then per-node breakdowns. Summary line at the bottom.
- **JSON**: Structured output with top-level keys: `global`, `nodes` (array), `unjoined` (array), `errors` (array).

**Rationale**: Matches Constitution Principle V. Human-readable for interactive use; JSON for scripting. Event-level trace references (event IDs) are included in JSON output by default but NOT in the default human-readable output (too verbose). Human-readable output includes a summary count of source events.

**Alternatives considered**:
- Always include event IDs in human-readable output — rejected: would make output unwieldy for large datasets
- Separate `--verbose` flag for trace details — viable future enhancement if users request it

## Decision: Handling unmatched kind 8383 events

**Chosen**: Kind 8383 events with no matching kind 38383 are tracked in an `unjoined` list. They are:
- **Excluded** from all volume and average statistics (sats volume, fiat volume, average order size)
- **Included** in a separate "unjoined events" count reported at the bottom of the output
- **NOT** contributing to fee-related counts in the main statistics — development fees are only counted for successfully joined records

**Rationale**: The spec states kind 8383 is the entry point, and statistics represent trading activity. An unmatched kind 8383 event represents a fee payment where the order details are unknown. Including its fee data in statistics would be an inferred metric (violating Constitution Principle I — Correctness Over Cleverness). The event is reported explicitly so the user knows it was seen but excluded from aggregates.

**Alternatives considered**:
- Count fees from unmatched 8383 events — rejected: would be a partial statistic from incomplete data
- Silently ignore unmatched events — rejected: violates Constitution Principle IV (Transparent Handling of Incomplete Data)

## Decision: `y` tag display when missing or inconsistent

**Chosen**: The `y` tag's third value (node display name) is collected from kind 8383 events per node. The second `y` tag value (`"mostro"`) is a fixed platform marker and is NOT exposed in reports. When displaying per-node statistics:
- If **all** events for a given node pubkey have the same `y` tag third value → display that value
- If events for the same node pubkey have **different** `y` tag third values → display the value from the most recently seen kind 8383 event (highest `created_at` for that node pubkey). No reconciliation or frequency-based strategy is applied in v1.
- If **no** events for a node have a `y` tag third value → `name` is empty (no `"N/A"` placeholder)

**Rationale**: The `y` tag follows the Mostro platform convention. The second element is a fixed marker (`"mostro"`); the optional third element provides the node's chosen display name. Using the most recent value is simple and deterministic; no invented reconciliation strategy is applied in v1.

**Alternatives considered**:
- Show first seen value — rejected: not representative if inconsistent
- Show all unique values — rejected: would clutter output

## Decision: Deduplication of kind 8383 events

**Chosen**: Deduplication is based on Nostr event ID (the hash of the event). If multiple events have the same content but different event IDs, they are treated as distinct. If multiple events share the same event ID (exact duplicate from relay re-delivery), only one is processed.

**Rationale**: Nostr event IDs are cryptographic hashes; same ID means identical event. This is the standard Nostr deduplication approach.

## Decision: Fiat volume grouping — no cross-currency totals

**Chosen**: Fiat volume is reported separately per fiat currency. There is no cross-currency grand total. For example:

```
Fiat Volume:
  USD: 50,000
  EUR: 30,000
  ARS: 5,000,000
```

No sum or equivalent conversion is computed in v1.

**Rationale**: The spec explicitly requires "summing fiat amounts separately per fiat currency, with no cross-currency grand total." This avoids the complexity and inaccuracy of currency conversion in v1.

## Decision: Node identity

**Chosen**: A Mostro node is identified by the author pubkey of the kind 8383 event. The kind 38383 event is treated purely as an enrichment source for order metadata (amounts, fiat, side), not as a source of node identity. The author pubkey of the kind 38383 event is not used for node identification in v1.

**Rationale**: The spec clarifies that node identity is anchored to kind 8383 author pubkey. This is simpler and unambiguous — kind 8383 is the entry point.

## Decision: Traceability output scope

**Chosen**:
- **JSON output**: Always includes `source_event_ids` arrays. Each aggregated statistic section (global and per-node) includes an array of kind 8383 and kind 38383 event IDs that contributed to that statistic. This is not gated behind a debug flag — it is always present in JSON output because JSON is designed for machine consumption and audit.
- **Human-readable output (default)**: Does NOT include event IDs. Includes only the data quality summary counts (processed/joined/unmatched/skipped) and a total source events line (e.g., "Processed 1,234 kind 8383 events, joined with 1,180 kind 38383 events").

**Rationale**: Including event IDs in human-readable output would make it unreadable for large datasets. JSON output is always machine-oriented, so event IDs are always included there. The spec requires traceability (FR-018) — JSON output is the mechanism for audit.

## Decision: Fiat currency normalization

**Chosen**: All fiat currency codes from kind 38383 events are normalized to uppercase during parsing. Matching, grouping, and filtering all operate on the uppercase form. For example, "usd", "Usd", and "USD" all normalize to "USD" and are grouped together.

**Rationale**: Relay data may contain currency codes in any case. Case-insensitive normalization prevents artificial splitting of the same currency into separate groups (e.g., "usd" and "USD" appearing as two different currencies). Uppercase is the conventional display form for ISO 4217 currency codes.

## Decision: Data quality summary structure

**Chosen**: A stable four-count summary is always present in both human-readable and JSON output:
- **processed**: total kind 8383 events successfully parsed
- **joined**: kind 8383 events matched to kind 38383 events (contributing to statistics)
- **unmatched**: kind 8383 events with no corresponding kind 38383
- **skipped**: kind 8383 events skipped due to malformed data (missing `order-id`, invalid tags)

The invariant `processed == joined + unmatched + skipped` MUST hold. If it does not, the tool MUST exit with a non-zero status (this is an internal bug, not a user-facing error).

**Rationale**: The constitution requires transparent handling of incomplete data (Principle IV). This summary gives users an immediate, unambiguous picture of data completeness. The invariant provides a self-check mechanism.

## Decision: Protocol tag sources for key fields

**Chosen**:
- **Development fee amount** (kind 8383): extracted from the `amount` tag. The value is an integer representing satoshis. If the `amount` tag is absent or non-numeric, the event is classified as **skipped** in the data quality summary.
- **Order side** (kind 38383): extracted from the `type` tag. Values are compared case-insensitively and normalized to `Buy` or `Sell`. Unrecognized or absent values map to `Unknown` and are excluded from side-filtered queries and side-grouped statistics.

**Rationale**: These are the canonical tag names defined in the Mostro protocol specification. Documenting them explicitly prevents implementation guesswork.

**Protocol references**:
- Kind 8383 `amount` tag: <https://mostro.network/protocol/other_events.html#development-fee>
- Kind 38383 `type` tag: <https://mostro.network/protocol/order_event.html>
