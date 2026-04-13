use serde::Serialize;
use std::collections::HashMap;

// ── Raw event ──────────────────────────────────────────────────────────────

/// Thin wrapper around a nostr-sdk event as received from the relay.
#[derive(Debug, Clone)]
pub struct NostrEvent {
    pub id: String,
    pub kind: u16,
    pub pubkey: String,
    pub created_at: u64,
    pub tags: Vec<Vec<String>>,
    pub content: String,
}

// ── Parsed kind 8383 (development fee) ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DevFeeEvent {
    pub event_id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub order_id: String,
    pub y_tag_value: Option<String>,
    pub fee_amount_sats: u64,
}

// ── Parsed kind 38383 (order) ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub event_id: String,
    pub d_tag: String,
    pub amount_sats: u64,
    pub fiat_currency: Option<String>,
    pub fiat_amount: Option<f64>,
    pub order_side: Option<OrderSide>,
}

// ── Order side ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum OrderSide {
    Buy,
    Sell,
    Unknown,
}

impl OrderSide {
    /// Parse from the `type` tag value, case-insensitive.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => OrderSide::Unknown,
        }
    }
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
            OrderSide::Unknown => write!(f, "unknown"),
        }
    }
}

// ── Joined record ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JoinedOrderRecord {
    pub fee_event: DevFeeEvent,
    pub order_event: OrderEvent,
    pub node_pubkey: String,
    pub order_id: String,
}

// ── Unjoined record ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub enum UnjoinReason {
    #[serde(rename = "OrderNotFound")]
    OrderNotFound,
    #[serde(rename = "OrderMalformed")]
    OrderMalformed,
    #[serde(rename = "MalformedFeeEvent")]
    MalformedFeeEvent,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnjoinedRecord {
    pub event_id: String,
    pub order_id: String,
    pub reason: UnjoinReason,
}

// ── Node identity key ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeKey {
    pub pubkey: String,
    pub y_tag_value: Option<String>,
}

// ── Per-node stats ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct NodeStats {
    pub node: NodeKeySerde,
    pub order_count: u64,
    pub total_fees_sats: u64,
    pub total_volume_sats: u64,
    pub avg_order_size_sats: f64,
    pub fiat_volume_by_currency: HashMap<String, f64>,
    pub volume_by_side: HashMap<String, u64>,
    pub source_event_ids: Vec<String>,
    #[serde(skip)]
    pub _last_y_tag_created_at: u64,
}

/// Serialisable form of NodeKey (avoids deriving Serialize on the real type
/// which carries Option that serde handles fine, but we keep explicit struct).
#[derive(Debug, Clone, Serialize)]
pub struct NodeKeySerde {
    pub pubkey: String,
    pub y_tag_value: Option<String>,
}

impl From<&NodeKey> for NodeKeySerde {
    fn from(nk: &NodeKey) -> Self {
        NodeKeySerde {
            pubkey: nk.pubkey.clone(),
            y_tag_value: nk.y_tag_value.clone(),
        }
    }
}

// ── Global stats ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct GlobalStats {
    pub order_count: u64,
    pub total_fees_sats: u64,
    pub total_volume_sats: u64,
    pub avg_order_size_sats: f64,
    pub fiat_volume_by_currency: HashMap<String, f64>,
    pub volume_by_side: HashMap<String, u64>,
    pub source_event_ids: Vec<String>,
}

// ── Data quality summary ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DataQualitySummary {
    pub processed: u64,
    pub joined: u64,
    pub unmatched: u64,
    pub skipped: u64,
}

// ── Complete report output ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ReportOutput {
    pub global: GlobalStats,
    pub nodes: Vec<NodeStats>,
    pub data_quality: DataQualitySummary,
    pub unjoined: Vec<UnjoinedRecord>,
    pub errors: Vec<String>,
    pub filter_summary: String,
}
