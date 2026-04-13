# Data Model: Mostro Stats CLI

**Date**: 2026-04-13
**Feature**: 001-mostro-stats-cli

## Entities

### NostrEvent (Raw)

A raw event as received from the relay. This is a thin wrapper around nostr-sdk's event type.

**Fields**:
- `id`: String — Nostr event ID (hex)
- `kind`: u16 — Event kind number (8383 or 38383)
- `pubkey`: String — Author's public key (hex)
- `created_at`: u64 — Unix timestamp of event creation
- `tags`: Vec<Tag> — Nostr tags array
- `content`: String — Event content (not used in v1 for kind 8383/38383)

**Validation rules**:
- `id` must be a valid 32-byte hex string (64 characters)
- `pubkey` must be a valid 32-byte hex string (64 characters)
- `tags` must be parseable as key-value pairs

### DevFeeEvent (Kind 8383 Parsed)

A parsed kind 8383 development fee event.

**Fields**:
- `event_id`: String — Nostr event ID
- `pubkey`: String — Author pubkey (identifies the Mostro node)
- `created_at`: u64 — Event timestamp (used for date filtering)
- `order_id`: String — Value of the `order-id` tag
- `y_tag_value`: Option<String> — Second value of the `y` tag, if present
- `fee_amount_sats`: u64 — Development fee amount in satoshis (extracted from event tags)

**Validation rules**:
- `order_id` MUST be present; events without it are flagged as unprocessable
- `fee_amount_sats` MUST be a positive integer; zero-fee events are logged but processed
- `created_at` MUST be a valid Unix timestamp

**Source**: Parsed from NostrEvent of kind 8383. Protocol reference: <https://mostro.network/protocol/other_events.html#development-fee>

### OrderEvent (Kind 38383 Parsed)

A parsed kind 38383 order event.

**Fields**:
- `event_id`: String — Nostr event ID
- `d_tag`: String — The `d` tag value (matches `order_id` from DevFeeEvent)
- `amount_sats`: u64 — Total order amount in satoshis
- `fiat_currency`: Option<String> — Fiat currency code, **normalized to uppercase** (e.g., "usd" → "USD"). Stored as uppercase; normalization occurs during parsing.
- `fiat_amount`: Option<f64> — Order amount in fiat currency
- `order_side`: Option<OrderSide> — Buy or Sell

**Validation rules**:
- `d_tag` MUST be present; events without it are unprocessable
- `amount_sats` MUST be a positive integer
- `fiat_currency` and `fiat_amount` may both be absent (fiat-less orders are valid)
- If `fiat_currency` is present but `fiat_amount` is absent (or vice versa), the event is processed for sats but fiat is excluded from fiat volume

**Source**: Parsed from NostrEvent of kind 38383. Protocol reference: <https://mostro.network/protocol/order_event.html>

### OrderSide (Enum)

Order direction.

**Values**:
- `Buy`
- `Sell`
- `Unknown` — When the tag is absent or unrecognized

### JoinedOrderRecord

The result of joining a DevFeeEvent with its corresponding OrderEvent.

**Fields**:
- `fee_event`: DevFeeEvent — The kind 8383 event
- `order_event`: OrderEvent — The matched kind 38383 event
- `node_pubkey`: String — The author pubkey from the fee event (node identity)
- `order_id`: String — The shared order ID

**Join rule**: `DevFeeEvent.order_id == OrderEvent.d_tag`

**Uniqueness**: Multiple fee events may reference the same order ID. Each fee event produces a separate JoinedOrderRecord even if the order event is shared.

### UnjoinedRecord

A kind 8383 event that could not be joined to a kind 38383 event.

**Fields**:
- `fee_event`: DevFeeEvent — The kind 8383 event
- `reason`: UnjoinReason — Why the join failed
  - `OrderNotFound` — No kind 38383 event exists with matching `d` tag
  - `OrderMalformed` — Kind 38383 event exists but is missing required fields
  - `MalformedFeeEvent` — Kind 8383 event has missing or invalid `order-id` tag

### NodeKey

Composite identifier for display purposes.

**Fields**:
- `pubkey`: String — The node's author pubkey
- `y_tag_value`: Option<String> — The `y` tag second value (resolved per-node across all events)

**Resolution**: For a given pubkey, the `y_tag_value` is resolved across all DevFeeEvents for that node using the rule from research.md:
- If all events agree → use that value
- If events disagree → use most frequent value with count indicator
- If no events have the value → `None`

## Aggregation Structures

### NodeStats

Per-node aggregated statistics.

**Fields**:
- `node`: NodeKey — Node identity
- `order_count`: u64 — Number of joined orders
- `total_fees_sats`: u64 — Sum of development fees
- `total_volume_sats`: u64 — Sum of order amounts in sats
- `avg_order_size_sats`: f64 — total_volume_sats / order_count
- `fiat_volume_by_currency`: HashMap<String, f64> — Fiat amounts grouped by currency code
- `volume_by_side`: HashMap<OrderSide, u64> — Sats volume grouped by order side
- `source_event_ids`: Vec<String> — Event IDs for traceability (JSON output only)

### GlobalStats

Network-wide aggregated statistics.

**Fields**:
- `order_count`: u64 — Total number of joined orders across all nodes
- `total_fees_sats`: u64 — Sum of all development fees
- `total_volume_sats`: u64 — Sum of all order amounts in sats
- `avg_order_size_sats`: f64 — total_volume_sats / order_count
- `fiat_volume_by_currency`: HashMap<String, f64> — Fiat amounts grouped by currency code
- `volume_by_side`: HashMap<OrderSide, u64> — Sats volume grouped by order side
- `source_event_ids`: Vec<String> — Event IDs for traceability (JSON output only)

### ReportOutput

The complete output of a successful run.

**Fields**:
- `global`: GlobalStats — Network-wide statistics
- `nodes`: Vec<NodeStats> — Per-node statistics (sorted by order_count descending)
- `data_quality`: DataQualitySummary — Stable four-count summary (always present)
- `unjoined`: Vec<UnjoinedRecord> — Details (JSON output only)
- `errors`: Vec<String> — Processing errors (JSON output only)
- `filter_summary`: String — Human-readable description of active filters

### DataQualitySummary

Invariant: `processed == joined + unmatched + skipped`.

**Fields**:
- `processed`: u64 — Total kind 8383 events successfully parsed
- `joined`: u64 — Kind 8383 events matched to kind 38383 events (contributing to statistics)
- `unmatched`: u64 — Kind 8383 events with no corresponding kind 38383 on the relay
- `skipped`: u64 — Kind 8383 events skipped due to malformed data (missing `order-id`, invalid tags)

## Relationships

```
NostrEvent (kind 8383)  →  DevFeeEvent  ──join by order_id──→  OrderEvent  ←── NostrEvent (kind 38383)
                                      │
                                      └──→  JoinedOrderRecord
                                                │
                                                ├──→ NodeStats (grouped by node_pubkey)
                                                └──→ GlobalStats (aggregate of all JoinedOrderRecords)
```

## Filter Application

Filters are applied as predicates on the collection of JoinedOrderRecords before aggregation:

- **Date range**: `fee_event.created_at >= from_timestamp AND fee_event.created_at <= to_timestamp`
- **Node pubkey**: `node_pubkey == target_pubkey`
- **Fiat currency**: `order_event.fiat_currency == target_currency` (excludes orders with no fiat or different currency)
- **Order side**: `order_event.order_side == target_side` (excludes orders with unknown side)

All active filters are AND-composed.

## Data Flow

1. Fetch raw kind 8383 events → parse to DevFeeEvents (skip malformed → counted as **skipped**)
2. Deduplicate order IDs from DevFeeEvents (unique `order_id` values, preserving event-level multiplicity)
3. Fetch kind 38383 events in a **single batched relay query** filtering on `d` tags matching all unique order IDs
4. Parse kind 38383 events to OrderEvents; normalize `fiat_currency` to uppercase
5. Join by order_id → produce JoinedOrderRecords + UnjoinedRecords (unmatched → counted as **unmatched**)
6. Apply filters → filtered JoinedOrderRecords
7. Aggregate → GlobalStats + Vec<NodeStats>
8. Format → ReportOutput (human-readable or JSON) with invariant `processed == joined + unmatched + skipped`
