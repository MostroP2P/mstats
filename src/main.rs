#![allow(dead_code)]
#![allow(clippy::new_without_default)]

pub mod aggregator;
mod cli;
mod config;
pub mod event_parser;
pub mod filters;
pub mod joiner;
mod models;
pub mod output;
mod relay;

use aggregator::aggregate;
use cli::Cli;
use config::Config;
use event_parser::{parse_dev_fee_event, parse_order_event};
use filters::apply_filters;
use joiner::join_events;
use output::{print_human_readable, print_json};
use relay::RelayClient;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = Cli::parse_args();
    let config = Config::new();
    let client = RelayClient::new(config);

    // Phase A: Fetch kind 8383 events
    let fee_events_raw = client.fetch_kind_8383_events().await?;

    if fee_events_raw.is_empty() {
        println!("No kind 8383 events found for the specified query.");
        return Ok(());
    }

    // Parse kind 8383 events (track skipped count)
    let mut skipped_count: u64 = 0;
    let fee_events: Vec<models::DevFeeEvent> = fee_events_raw
        .into_iter()
        .filter_map(|ev| match parse_dev_fee_event(&ev) {
            Ok(parsed) => Some(parsed),
            Err(_) => {
                skipped_count += 1;
                None
            }
        })
        .collect();

    // Phase B: Fetch kind 38383 events with the same windowed strategy and filter locally
    let order_events_raw = client.fetch_kind_38383_events().await?;

    let unique_order_ids: std::collections::HashSet<String> =
        fee_events.iter().map(|e| e.order_id.clone()).collect();

    let order_events: Vec<models::OrderEvent> = order_events_raw
        .into_iter()
        .filter_map(|ev| parse_order_event(&ev).ok())
        .filter(|ev| unique_order_ids.contains(&ev.d_tag))
        .collect();

    // Join
    let (joined, unjoined) = join_events(&fee_events, &order_events);

    // Apply filters
    let from_ts = args
        .from
        .as_ref()
        .map(|s| parse_timestamp(s, false))
        .transpose()?;
    let to_ts = args
        .to
        .as_ref()
        .map(|s| parse_timestamp(s, true))
        .transpose()?;
    let node_pubkey = args.node.as_deref();
    let currency = args.currency.as_deref();
    let side = args.side.as_deref();

    let filtered = apply_filters(joined, from_ts, to_ts, node_pubkey, currency, side);

    // Build filter summary
    let filter_summary = build_filter_summary(from_ts, to_ts, node_pubkey, currency, side);

    // Aggregate
    let mut report = aggregate(filtered, unjoined, skipped_count);
    report.filter_summary = filter_summary;

    // Output
    if args.json {
        print_json(&report);
    } else {
        print_human_readable(&report);
    }

    Ok(())
}

/// Parse a date string (ISO 8601 or Unix timestamp) to a Unix timestamp.
fn parse_timestamp(s: &str, end_of_day: bool) -> Result<u64, String> {
    if let Ok(ts) = s.parse::<u64>() {
        return Ok(ts);
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = if end_of_day {
            date.succ_opt()
                .and_then(|next| next.and_hms_opt(0, 0, 0))
                .ok_or_else(|| format!("Invalid date format '{}'", s))?
                - chrono::TimeDelta::seconds(1)
        } else {
            date.and_hms_opt(0, 0, 0)
                .ok_or_else(|| format!("Invalid date format '{}'", s))?
        };
        return Ok(dt.and_utc().timestamp() as u64);
    }

    let dt: chrono::DateTime<chrono::Utc> = s
        .parse()
        .map_err(|e| format!("Invalid date format '{}': {}", s, e))?;
    Ok(dt.timestamp() as u64)
}

/// Build a human-readable filter summary string.
fn build_filter_summary(
    from_ts: Option<u64>,
    to_ts: Option<u64>,
    node_pubkey: Option<&str>,
    currency: Option<&str>,
    side: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let (Some(from), Some(to)) = (from_ts, to_ts) {
        parts.push(format!("date: {} to {}", format_ts(from), format_ts(to)));
    } else if let Some(from) = from_ts {
        parts.push(format!("from: {}", format_ts(from)));
    } else if let Some(to) = to_ts {
        parts.push(format!("to: {}", format_ts(to)));
    }
    if let Some(pk) = node_pubkey {
        let short_pk: String = pk.chars().take(16).collect();
        parts.push(format!("node: {}...", short_pk));
    }
    if let Some(cur) = currency {
        parts.push(format!("currency: {}", cur.to_uppercase()));
    }
    if let Some(s) = side {
        parts.push(format!("side: {}", s));
    }
    if parts.is_empty() {
        "No filters applied".to_string()
    } else {
        format!("Filters: {}", parts.join(", "))
    }
}

fn format_ts(ts: u64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| ts.to_string())
}
