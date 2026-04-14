use clap::Parser;

/// mstats — Mostro trading statistics CLI
///
/// Fetches kind 8383 development fee events and kind 38383 order events from
/// wss://relay.mostro.network, joins them by order ID, and produces global
/// and per-node aggregated statistics.
#[derive(Parser, Debug)]
#[command(name = "mstats", version, about, long_about = None)]
pub struct Cli {
    /// Output in JSON format instead of human-readable text
    #[arg(long)]
    pub json: bool,

    /// Start of date range (inclusive). ISO 8601 date or Unix timestamp.
    /// Applies to kind 8383 `created_at` timestamp.
    #[arg(long)]
    pub from: Option<String>,

    /// End of date range (inclusive). ISO 8601 date or Unix timestamp.
    /// Date-only values interpreted as midnight UTC of the next day (exclusive upper bound).
    #[arg(long)]
    pub to: Option<String>,

    /// Filter to a specific Mostro node by its hex pubkey (64 hex chars).
    #[arg(long)]
    pub node: Option<String>,

    /// Filter to a specific fiat currency (e.g., USD, EUR).
    #[arg(long)]
    pub currency: Option<String>,

    /// Filter by order side: buy or sell.
    #[arg(long)]
    pub side: Option<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}
