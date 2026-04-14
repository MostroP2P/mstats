# Feature Specification: Mostro Stats CLI

**Feature Branch**: `001-mostro-stats-cli`
**Created**: 2026-04-13
**Status**: Draft
**Input**: User description: "Build a Rust CLI tool named mstats that analyzes Mostro trading activity using Nostr events."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run global and per-node statistics reports (Priority: P1)

A Mostro operator or auditor runs the `mstats` CLI to see aggregated trading statistics. The tool connects to the configured relay, fetches kind 8383 development fee events, extracts order IDs, fetches the corresponding kind 38383 order events, joins them by order ID, and outputs both global network-wide statistics and per-node reports. Global statistics cover all observed Mostro nodes; per-node statistics cover each individual node identified by the author pubkey of the kind 8383 event, displaying both the node pubkey and the second value of the `y` tag from the kind 8383 event when available.

Each report shows number of orders, total development fees paid, total traded volume in sats, average order size in sats, and fiat volume grouped by currency.

**Why this priority**: This is the core value proposition — the entire tool exists to produce these reports. Without them, there is no MVP.

**Independent Test**: Run the CLI against the relay and verify the output contains both global totals and per-node breakdowns with correct aggregations. Cross-check a sample of reported numbers against raw events fetched independently from the relay.

**Acceptance Scenarios**:

1. **Given** the relay is reachable and contains kind 8383 and kind 38383 events, **When** the user runs the CLI with default settings, **Then** a human-readable report is printed to stdout showing global network statistics followed by per-node statistics including node pubkey, `y` tag second value (when available), order count, total fees, total sats volume, average order size, and fiat volume by currency.
2. **Given** a specific order ID exists in a kind 8383 event, **When** the tool processes that event, **Then** it fetches the matching kind 38383 order event (matched by `d` tag equal to the order ID) and correctly joins the data for aggregation.
3. **Given** the user requests JSON output via a flag, **When** the tool runs, **Then** the output is valid JSON with the same statistical data structured for machine consumption.

---

### User Story 2 - Filter statistics by date, node, currency, and order side (Priority: P1)

A user needs to narrow the analysis to a specific date range, a particular Mostro node, a specific fiat currency, or a specific order side (buy/sell). The CLI must support filtering flags so users can view statistics from any combination of these dimensions.

**Why this priority**: Real-world analysis always requires slicing data by these dimensions. Without filtering, the tool only provides a single global view, which limits its usefulness for operators comparing their own activity or focusing on specific currencies.

**Independent Test**: Run the CLI with various filter combinations (e.g., date range + specific node, or currency + order side) and verify that reported statistics only include events matching all active filters.

**Acceptance Scenarios**:

1. **Given** the user specifies a date range, **When** the tool runs, **Then** only events within that date range are included in statistics.
2. **Given** the user specifies a Mostro node pubkey, **When** the tool runs, **Then** only statistics for that node are shown in the per-node report, and global statistics reflect only that node's activity.
3. **Given** the user specifies a fiat currency, **When** the tool runs, **Then** only orders with that fiat currency are included in the report.
4. **Given** the user specifies an order side (buy or sell), **When** the tool runs, **Then** only orders matching that side are included in the report.
5. **Given** the user combines multiple filters (e.g., date range + node + order side), **When** the tool runs, **Then** only events matching all specified filters are included.

---

### User Story 3 - Audit traceability of reported statistics (Priority: P1)

An auditor needs to verify that every number in the report is traceable to specific source events. The tool must make it possible to confirm that statistics are derived from real events, not inferred or fabricated.

**Why this priority**: The constitution requires correctness over cleverness and transparent handling of incomplete data. Traceability is the mechanism that makes correctness verifiable.

**Independent Test**: Select a reported statistic (e.g., "Node X paid Y sats in fees") and trace it back to the specific kind 8383 and kind 38383 event IDs that produced it.

**Acceptance Scenarios**:

1. **Given** a reported statistic in the output, **When** the user examines the tool's behavior, **Then** every number is directly attributable to one or more specific Nostr event IDs.
2. **Given** the tool encounters events that cannot be joined (e.g., a kind 8383 with no matching kind 38383), **When** the tool produces output, **Then** it clearly indicates which data was incomplete and excluded from statistics.

---

### User Story 4 - Handle relay errors and incomplete data gracefully (Priority: P2)

The tool encounters relay connectivity issues, malformed events, or events with missing tags. The user needs the tool to report what it could process and clearly indicate what was skipped or failed.

**Why this priority**: Real-world relay data is messy. The tool must never silently produce incorrect results when data is incomplete. However, the tool can still deliver value by reporting on the subset of data it successfully processed.

**Independent Test**: Simulate or observe relay failure, malformed events, or events missing the `order-id` tag. Verify the tool reports errors explicitly and does not fabricate statistics.

**Acceptance Scenarios**:

1. **Given** the relay is unreachable or times out, **When** the user runs the tool, **Then** the tool exits with a non-zero status and prints a clear error message to stderr.
2. **Given** some events are malformed or missing required tags, **When** the tool processes the remaining valid events, **Then** it reports statistics from valid data and explicitly lists which events were skipped and why.
3. **Given** a kind 8383 event has no matching kind 38383 event, **When** the tool aggregates statistics, **Then** that order ID is excluded from volume calculations and noted as an unjoined event.

---

### User Story 5 - Scripted analysis via JSON output (Priority: P2)

A developer or analyst pipes the tool's JSON output into another script or tool for further analysis, alerting, or archival.

**Why this priority**: Enables composability with downstream tools, but the tool is still useful without it.

**Independent Test**: Run the tool with a JSON output flag and verify the output is valid parseable JSON containing all statistical fields from the human-readable report, including both global and per-node sections.

**Acceptance Scenarios**:

1. **Given** the user passes a JSON output flag, **When** the tool runs successfully, **Then** stdout contains valid JSON that can be parsed by standard JSON tools.
2. **Given** the JSON output, **When** a script processes it, **Then** all statistics (global totals, per-node breakdowns with node pubkey and `y` tag values, order counts, fees, sats volume, average size, fiat volume, order side breakdowns) are accessible as structured fields.

---

### Edge Cases

- **What happens when** a kind 8383 event has no `order-id` tag? — The event is skipped and reported as unprocessable.
- **How does the system handle** a kind 8383 event referencing a kind 38383 event that does not exist on the relay? — The order is excluded from volume/fee aggregations and logged as an unjoined reference.
- **What happens when** the relay returns zero kind 8383 events for the query window? — The tool reports "no data found" and exits cleanly (non-error, since this is a valid state).
- **How does the system handle** duplicate kind 8383 events for the same order ID? — Each unique event is counted once; deduplication is based on event ID, not content.
- **What happens when** fiat currency fields in kind 38383 events are missing or unrecognized? — The event's sats data is still aggregated, but fiat volume is noted as "unspecified currency" or skipped with a warning.
- **What happens when** the `y` tag second value is missing from a kind 8383 event? — Per-node reports show the node pubkey with the `y` tag field marked as unavailable.
- **What happens when** multiple kind 8383 events for the same node pubkey expose different `y` tag second values? — In v1 the tool displays the `y` tag value from the most recently seen (highest `created_at`) kind 8383 event for that node. No reconciliation or frequency-based strategy is applied.
- **How does the system handle** orders where the `d` tag on the kind 38383 event does not match any `order-id` from kind 8383 events? — These orphan order events are not included in v1 statistics (kind 8383 is the entry point).
- **What happens when** a kind 8383 event cannot be joined to a kind 38383 event? — The unmatched event is reported in the unjoined list with all directly available kind 8383 metadata: `event_id`, `order_id` (when present), `pubkey` (node identity), `y` tag second value (when available), and `amount` tag value (when parseable). No order-level fields are fabricated; the reason for the failed join is included.
- **What happens when** the data quality summary counts do not add up (processed ≠ joined + unmatched + skipped)? — This is an internal invariant violation; the tool MUST NOT produce output and MUST exit with a non-zero status.

### Data Quality Summary

Every run MUST report a stable four-count summary:
- **Processed**: total kind 8383 events successfully parsed
- **Joined**: kind 8383 events successfully matched to kind 38383 events (contributing to statistics)
- **Unmatched**: kind 8383 events with no corresponding kind 38383 on the relay
- **Skipped**: kind 8383 events skipped due to malformed data (missing `order-id` tag, invalid tags)

Invariant: `processed == joined + unmatched + skipped`. This count set is always present in both human-readable and JSON output.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tool MUST fetch kind 8383 development fee events from `wss://relay.mostro.network`.
- **FR-002**: The tool MUST extract the `order-id` tag from each kind 8383 event.
- **FR-003**: The tool MUST fetch kind 38383 order events from the same relay for every unique order ID extracted from kind 8383 events. Kind 38383 fetching is mandatory — the tool MUST NOT produce statistics without it.
- **FR-003a**: The tool MUST deduplicate order IDs extracted from kind 8383 events before querying the relay. Kind 38383 events MUST be fetched in a single batched relay query using a filter on the `d` tags matching the deduplicated order IDs, NOT with one relay roundtrip per order.
- **FR-004**: The tool MUST join kind 8383 and kind 38383 events by order ID.
- **FR-005**: The tool MUST compute global (network-wide) statistics across all observed Mostro nodes.
- **FR-006**: The tool MUST compute per-node statistics, where a node is identified by the author pubkey of the kind 8383 event.
- **FR-007**: Per-node reports MUST display the node pubkey and the second value of the `y` tag from the kind 8383 event when available.
- **FR-008**: The tool MUST report the following statistics for each aggregation scope (global and per-node):
  - Number of orders
  - Total development fees paid
  - Total traded volume in sats
  - Average order size in sats
  - Fiat volume grouped by fiat currency
- **FR-009**: The tool MUST extract and use the order side (buy/sell) from kind 38383 events for filtering and reporting.
- **FR-010**: The tool MUST support filtering statistics by date range.
- **FR-011**: The tool MUST support filtering statistics by Mostro node pubkey.
- **FR-012**: The tool MUST support filtering statistics by fiat currency. Currency matching MUST be case-insensitive, with all currency codes normalized to uppercase before comparison (e.g., "usd", "Usd", and "USD" all match "USD").
- **FR-013**: The tool MUST support filtering statistics by order side (buy/sell).
- **FR-014**: Multiple filters MUST be composable — all active filters are applied simultaneously.
- **FR-015**: The tool MUST provide human-readable output by default.
- **FR-016**: The tool MUST support optional JSON output via a CLI flag.
- **FR-017**: The tool MUST explicitly report events that could not be processed (missing tags, no matching kind 38383, malformed data). Both human-readable and JSON output MUST include a stable data quality summary section with these four counts: (1) **processed** — total kind 8383 events successfully parsed, (2) **joined** — kind 8383 events successfully matched to kind 38383 events, (3) **unmatched** — kind 8383 events with no corresponding kind 38383, (4) **skipped** — kind 8383 events skipped due to malformed data (missing `order-id`, missing pubkey, invalid tags). These counts MUST be consistent: `processed == joined + unmatched + skipped`.
- **FR-018**: Every statistic in the output MUST be directly traceable to specific source event IDs.
- **FR-019**: The tool MUST connect to a single relay only (`wss://relay.mostro.network`) in v1.
- **FR-020**: The tool MUST exit with a non-zero status code and print an error to stderr when the relay is unreachable or the query fails entirely.

**Protocol References**:
- Development fee event (kind 8383): <https://mostro.network/protocol/other_events.html#development-fee>
  - Development fee amount: extracted from the `amount` tag value in the kind 8383 event (integer satoshis). If the `amount` tag is absent or non-numeric, the event is classified as **skipped** in the data quality summary.
- Order event (kind 38383): <https://mostro.network/protocol/order_event.html>
  - Order side (buy/sell): extracted from the `type` tag in the kind 38383 event. The value is compared case-insensitively and normalized to `Buy` or `Sell`. Unrecognized or absent `type` values map to `Unknown` and are excluded from side-filtered queries and side-grouped statistics.

### Key Entities

- **Kind 8383 Event (Development Fee)**: A Nostr event representing a development fee payment made by a Mostro node to the Mostro development fund. Contains an `order-id` tag linking it to a specific order, and a `y` tag whose second value provides additional node identification. The author pubkey of this event identifies the Mostro node. Protocol reference: <https://mostro.network/protocol/other_events.html#development-fee>
- **Kind 38383 Event (Order)**: A Nostr event representing a Mostro order. Identified by its `d` tag (which matches the `order-id` from the corresponding kind 8383 event). Contains total order amount in sats, fiat currency code, fiat amount, and order side (buy/sell). Protocol reference: <https://mostro.network/protocol/order_event.html>
- **Mostro Node**: An independent Mostro operator identified by the author pubkey of the kind 8383 events they publish. All nodes publish events to Nostr; the tool observes and aggregates across them.
- **Joined Order Record**: The result of joining a kind 8383 event with its corresponding kind 38383 event by matching the `order-id` tag to the `d` tag. Contains fee data, order amount, fiat details, the node pubkey (from the kind 8383 event), `y` tag value, and order side.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can run the CLI and receive complete global and per-node statistics reports within 60 seconds for up to 10,000 kind 8383 events on the relay.
- **SC-002**: 100% of reported statistics are traceable to specific source event IDs — zero inferred or fabricated numbers.
- **SC-003**: The tool correctly joins kind 8383 and kind 38383 events for all matching pairs — verified by cross-referencing a random sample of 50 joined records against raw relay data with 100% accuracy.
- **SC-004**: When data is incomplete (unjoined events, missing tags, relay errors), the tool explicitly reports the gap in output — verified by test scenarios with known incomplete data.
- **SC-005**: JSON output is valid and parseable by standard JSON tools in 100% of successful runs.
- **SC-006**: The tool exits cleanly (status 0) with "no data" message when zero matching events exist, and exits with non-zero status on total relay failure.
- **SC-007**: Each filter (date, node, currency, order side) and every combination of filters correctly narrows the dataset — verified by running the tool with known subsets and confirming reported totals match the expected subset.
- **SC-008**: Global statistics equal the sum of all per-node statistics — verified for order count, total fees, and total sats volume with zero discrepancy.

## Assumptions

- The relay `wss://relay.mostro.network` is accessible and contains a representative set of kind 8383 and kind 38383 events for the target analysis period.
- For any kind 8383 event whose `order-id` tag equals `X` and any kind 38383 event whose `d` tag equals `X`, the two events are attributed to the same Mostro node (identified by the kind 8383 author pubkey).
- The `order-id` tag in kind 8383 events uses a consistent format that can be matched byte-for-byte against the `d` tag in kind 38383 events; mismatches on format (not just value) are treated the same as a missing counterpart.
- Fiat currency codes in kind 38383 events may appear in any case; the tool normalizes all codes to uppercase for matching, grouping, and filtering.
- Users have basic familiarity with running CLI tools and interpreting tabular/statistical output.
- No authentication or authorization is required to query events from the relay for v1.
- The analysis period (date range) defaults to all available events when no date filter is specified.
- The `y` tag on kind 8383 events may or may not be present; when present, its second value is used for display in per-node reports.
- Order side (buy/sell) is reliably encoded in kind 38383 events per the Mostro protocol specification.
