# Tasks: Mostro Stats CLI

**Input**: Design documents from `/specs/001-mostro-stats-cli/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Unit tests are included to ensure correctness of parsing, joining, aggregation, filtering, and output formatting. Integration tests are feature-gated and require live relay access.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
src/          # Source code (flat module layout)
tests/unit/   # Offline unit tests
tests/integration/  # Relay-dependent integration tests (feature-gated)
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize Rust 2021 edition project with `cargo init --name mstats`
- [x] T002 [P] Add dependencies to Cargo.toml: `nostr-sdk = "=0.44.1"`, `clap`, `serde`, `serde_json`, `chrono`
- [x] T003 [P] Configure `rustfmt` defaults and `clippy` linting in `.cargo/config.toml`
- [x] T004 Create `tests/unit/` and `tests/integration/` directory structure
- [x] T005 [P] Define `integration-tests` feature flag in Cargo.toml for gated integration tests

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 [P] Define core data structures in `src/models.rs`: `NostrEvent`, `DevFeeEvent`, `OrderEvent`, `OrderSide` enum, `JoinedOrderRecord`, `UnjoinedRecord`, `NodeKey`
- [x] T007 [P] Define aggregation structures in `src/models.rs`: `NodeStats`, `GlobalStats`, `DataQualitySummary`, `ReportOutput`
- [x] T008 Implement `config.rs` with hardcoded relay URL (`wss://relay.mostro.network`) and default settings
- [x] T009 Implement `relay.rs`: relay client wrapper using nostr-sdk 0.44.1 — connect, subscribe for events, collect all events with timeout, disconnect
- [x] T010 Implement `cli.rs`: clap derive-mode CLI definition with `--json`, `--from`, `--to`, `--node`, `--currency`, `--side` flags per CLI contract
- [x] T011 [P] Create `src/main.rs` entry point wiring CLI parsing → config → placeholder pipeline → output dispatch

**Checkpoint**: Foundation ready — user story implementation can now begin in parallel

---

## Phase 3: User Story 1 — Run global and per-node statistics reports (Priority: P1) 🎯 MVP

**Goal**: Fetch kind 8383 and kind 38383 events, join by order ID, compute global and per-node statistics, output human-readable report

**Independent Test**: Run the CLI against the relay (or mock data) and verify the output contains global totals and per-node breakdowns with correct aggregations. Cross-check a sample of reported numbers against raw events fetched independently.

### Tests for User Story 1

 - [x] T012 [P] [US1] Unit test: parse kind 8383 event with valid `order-id` and `amount` tags in `tests/unit/test_event_parser.rs`
 - [x] T013 [P] [US1] Unit test: parse kind 38383 event with valid `d`, `amount_sats`, `fiat_currency`, `fiat_amount`, `type` tags in `tests/unit/test_event_parser.rs`
 - [x] T014 [US1] Unit test: join DevFeeEvent + OrderEvent by matching order_id to d_tag in `tests/unit/test_joiner.rs`
 - [x] T015 [US1] Unit test: aggregate joined records into GlobalStats in `tests/unit/test_aggregator.rs`
 - [x] T016 [US1] Unit test: aggregate joined records into per-node NodeStats grouped by pubkey in `tests/unit/test_aggregator.rs`
 - [x] T017 [US1] Unit test: human-readable output formatter in `tests/unit/test_output.rs`

### Implementation for User Story 1

 - [x] T018 [US1] Implement `src/event_parser.rs`: parse kind 8383 → `DevFeeEvent` (extract `order-id`, `amount`, `y` tag 3rd value as `name`, pubkey, created_at); parse kind 38383 → `OrderEvent` (extract `d`, `amount_sats`, `fiat_currency` uppercase, `fiat_amount`, `type` → `OrderSide`)
 - [x] T019 [US1] Implement `src/relay.rs` relay fetch strategy using explicit time windows for kind 8383 and kind 38383 events, then perform local matching of `order-id` against `d`
 - [x] T020 [US1] Implement `src/joiner.rs`: match DevFeeEvent.order_id to OrderEvent.d_tag → produce `JoinedOrderRecord` and `UnjoinedRecord` (reason: `OrderNotFound`)
 - [x] T021 [US1] Implement `src/aggregator.rs`: compute `GlobalStats` and `Vec<NodeStats>` from `Vec<JoinedOrderRecord>` — order count, total fees, total sats volume, avg order size, fiat volume by currency, volume by side, `source_event_ids`
 - [x] T022 [US1] Implement `src/output.rs`: human-readable formatter printing global stats → per-node stats (sorted by order_count desc) → data quality summary
 - [x] T023 [US1] Wire main.rs pipeline: relay fetch → parse → join → aggregate → format (human-readable)

**Checkpoint**: At this point, `mstats` runs end-to-end: connects to relay, fetches both event kinds, joins, aggregates, and prints a correct human-readable report.

---

## Phase 4: User Story 2 — Filter statistics by date, node, currency, and order side (Priority: P1)

**Goal**: Support `--from`, `--to`, `--node`, `--currency`, `--side` flags that compose together to narrow statistics

**Independent Test**: Run the CLI with various filter combinations and verify reported statistics only include events matching all active filters.

### Tests for User Story 2

 - [x] T024 [P] [US2] Unit test: date range filter on `fee_event.created_at` in `tests/unit/test_filters.rs`
 - [x] T025 [P] [US2] Unit test: node pubkey filter in `tests/unit/test_filters.rs`
 - [x] T026 [P] [US2] Unit test: fiat currency filter (case-insensitive, normalized to uppercase) in `tests/unit/test_filters.rs`
 - [x] T027 [US2] Unit test: order side filter (Buy/Sell/Unknown) in `tests/unit/test_filters.rs`
 - [x] T028 [US2] Unit test: composed filters (date + node + side) in `tests/unit/test_filters.rs`
 - [x] T029 [US2] Unit test: `filter_summary` string reflects active filters in `tests/unit/test_output.rs`

### Implementation for User Story 2

 - [x] T030 [US2] Implement `src/filters.rs`: date range, node pubkey, fiat currency (uppercase-normalized), order side filter predicates on `Vec<JoinedOrderRecord>`; AND-compose active filters
 - [x] T031 [US2] Implement date parsing in `src/cli.rs`: ISO 8601 and Unix timestamp for `--from`/`--to`, with date-only semantics (from = midnight UTC inclusive, to = next-day midnight exclusive)
 - [x] T032 [US2] Wire filters into `src/main.rs`: apply filters after join, before aggregation
 - [x] T033 [US2] Update `src/output.rs`: include `filter_summary` in output describing active filters

**Checkpoint**: At this point, `mstats --from 2026-01-01 --currency USD --side buy` correctly narrows statistics. User Stories 1 and 2 both work independently.

---

## Phase 5: User Story 3 — Audit traceability of reported statistics (Priority: P1)

**Goal**: Every statistic in the output is directly traceable to specific source event IDs

**Independent Test**: Select a reported statistic and trace it back to the specific kind 8383 and kind 38383 event IDs that produced it.

### Tests for User Story 3

 - [x] T034 [P] [US3] Unit test: `source_event_ids` populated correctly in GlobalStats in `tests/unit/test_aggregator.rs`
 - [x] T035 [P] [US3] Unit test: `source_event_ids` populated correctly in each NodeStats in `tests/unit/test_aggregator.rs`
 - [x] T036 [US3] Unit test: JSON output includes `source_event_ids` arrays in `tests/unit/test_output.rs`

### Implementation for User Story 3

 - [x] T037 [US3] Update `src/aggregator.rs`: collect `source_event_ids` (kind 8383 + kind 38383 event IDs) per aggregation scope (global and per-node) during statistics computation

**Checkpoint**: At this point, JSON output includes event ID traces. User Stories 1–3 all work independently.

---

## Phase 6: User Story 4 — Handle relay errors and incomplete data gracefully (Priority: P2)

**Goal**: Relay failures, malformed events, and unjoined events are reported explicitly without fabricating statistics

**Independent Test**: Simulate relay failure, malformed events, or events missing required tags. Verify the tool reports errors explicitly and does not fabricate statistics.

### Tests for User Story 4

 - [x] T038 [P] [US4] Unit test: kind 8383 event missing `order-id` → classified as skipped in `tests/unit/test_event_parser.rs`
 - [x] T039 [P] [US4] Unit test: kind 8383 event with non-numeric `amount` tag → classified as skipped in `tests/unit/test_event_parser.rs`
 - [x] T040 [US4] Unit test: data quality summary invariant `processed == joined + unmatched + skipped` in `tests/unit/test_aggregator.rs`
 - [x] T041 [US4] Unit test: relay unreachable → exit code 1 + error to stderr in `tests/unit/test_relay.rs`
 - [x] T042 [US4] Unit test: zero kind 8383 events → exit code 0 + "no data" message in `tests/unit/test_relay.rs`

### Implementation for User Story 4

 - [x] T043 [US4] Update `src/event_parser.rs`: classify events with missing/invalid `order-id` or `amount` as skipped; count in `DataQualitySummary.skipped`
 - [x] T044 [US4] Update `src/joiner.rs`: record unjoined events with reasons (`OrderNotFound`, `OrderMalformed`); count in `DataQualitySummary.unmatched`
 - [x] T045 [US4] Update `src/aggregator.rs`: compute `DataQualitySummary` with `processed`, `joined`, `unmatched`, `skipped`; assert invariant
 - [x] T046 [US4] Update `src/output.rs`: include data quality summary section in both human-readable and JSON output
 - [x] T047 [US4] Update `src/relay.rs`: handle connection timeout/failure → exit code 1 + stderr error message; handle zero events → exit code 0 + "no data"

**Checkpoint**: At this point, the tool handles all error and incomplete-data paths gracefully.

---

## Phase 7: User Story 5 — Scripted analysis via JSON output (Priority: P2)

**Goal**: Provide valid JSON output with all statistical fields for machine consumption

**Independent Test**: Run the tool with `--json` flag and verify output is valid parseable JSON containing all statistical fields.

### Tests for User Story 5

 - [x] T048 [P] [US5] Unit test: JSON output is valid and parseable in `tests/unit/test_output.rs`
 - [x] T049 [US5] Unit test: JSON output contains all fields: `global`, `nodes`, `data_quality`, `unjoined`, `errors`, `filter_summary` in `tests/unit/test_output.rs`

### Implementation for User Story 5

 - [x] T050 [US5] Update `src/output.rs`: implement JSON output formatter using serde — serialize `ReportOutput` with `GlobalStats`, `Vec<NodeStats>`, `DataQualitySummary`, `Vec<UnjoinedRecord>`, `Vec<String>` errors, `filter_summary`
 - [x] T051 [US5] Wire `--json` flag in `src/main.rs`: dispatch to JSON formatter when flag is set

**Checkpoint**: All 5 user stories are independently functional.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

 - [x] T052 [P] Implement `name` resolution logic in `src/aggregator.rs`: most-recent-value-per-node (highest `created_at`), empty fallback
 - [x] T053 [P] Add fiat-less order handling: orders with missing `fiat_currency`/`fiat_amount` still contribute to sats volume but are excluded from fiat volume grouping
 - [x] T054 [P] Integration test: full pipeline against live relay in `tests/integration/test_relay_query.rs` (feature-gated)
 - [x] T055 Verify `cargo fmt --check`, `cargo clippy` (zero warnings), `cargo test` all pass
 - [x] T056 [P] Update `quickstart.md` with actual CLI examples after implementation
 - [x] T057 [P] Add `README.md` at repository root describing mstats purpose and usage

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (P1)**: Depends on Foundational phase — core pipeline (fetch, parse, join, aggregate, output)
- **User Story 2 (P1)**: Depends on Foundational phase — filter layer on top of joined records
- **User Story 3 (P1)**: Depends on User Story 1 — adds `source_event_ids` to aggregation
- **User Story 4 (P2)**: Depends on User Story 1 — error handling and data quality summary
- **User Story 5 (P2)**: Depends on User Story 1 — JSON output formatter
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Core pipeline — no story dependencies
- **User Story 2 (P1)**: Depends on Foundational; independent of US1 (can be developed after foundational, tested with mock joined data)
- **User Story 3 (P1)**: Depends on US1 aggregation (adds trace IDs to existing stats)
- **User Story 4 (P2)**: Depends on US1 pipeline (adds error handling paths)
- **User Story 5 (P2)**: Depends on US1 (adds JSON output mode to existing ReportOutput)

### Within Each User Story

- Models before services
- Services before pipeline wiring
- Tests before or alongside implementation
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T002, T003, T005)
- Foundational: T006, T007 (both `models.rs`) can run in parallel with T008 (config) and T009 (relay)
- US1 tests T012, T013 can run in parallel; T018 (parser) and T019 (relay fetch) are sequential within pipeline
- US2 tests T024, T025, T026 can run in parallel
- US3 tests T034, T035 can run in parallel
- US4 tests T038, T039 can run in parallel; T041, T042 can run in parallel
- US5 tests T048, T049 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all parsing tests together:
Task: "Unit test: parse kind 8383 event" (T012)
Task: "Unit test: parse kind 38383 event" (T013)

# Pipeline implementation:
Task: "Implement event_parser.rs" (T018) → "Implement relay.rs two-phase fetch" (T019) → "Implement joiner.rs" (T020) → "Implement aggregator.rs" (T021) → "Implement output.rs" (T022)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Run `cargo run --` against the relay; verify human-readable report is correct
5. Demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Run against relay → Verify correctness
3. Add User Story 2 → Test filter combinations → Verify narrowed statistics
4. Add User Story 3 → Verify `source_event_ids` in JSON output
5. Add User Story 4 → Test error paths (malformed events, relay failure, zero events)
6. Add User Story 5 → Verify JSON validity and completeness
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 2 (filters)
   - Developer B: User Story 4 (error handling)
   - Developer C: User Story 1 (core pipeline) — must complete first
3. After US1 completes:
   - Developer C: User Story 3 (traceability) + User Story 5 (JSON)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Constitution requires correctness over cleverness — verify statistics against raw relay data
- Integration tests require live relay connection; gate behind `integration-tests` feature flag
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
