use crate::models::{DevFeeEvent, NostrEvent, OrderEvent, OrderSide};

/// Parse a raw kind 8383 event into a DevFeeEvent.
///
/// Extracts:
/// - `order-id` tag → order_id
/// - `amount` tag → fee_amount_sats (integer satoshis)
/// - `y` tag 2nd value → y_tag_value
/// - event pubkey, created_at
pub fn parse_dev_fee_event(ev: &NostrEvent) -> Result<DevFeeEvent, String> {
    let order_id = find_tag_value(&ev.tags, "order-id")
        .ok_or_else(|| format!("Missing order-id tag in event {}", ev.id))?;

    let fee_amount_sats = find_tag_value(&ev.tags, "amount")
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or_else(|| format!("Missing or non-numeric amount tag in event {}", ev.id))?;

    let y_tag_value = find_tag_value_at_index(&ev.tags, "y", 1);

    Ok(DevFeeEvent {
        event_id: ev.id.clone(),
        pubkey: ev.pubkey.clone(),
        created_at: ev.created_at,
        order_id,
        y_tag_value,
        fee_amount_sats,
    })
}

/// Parse a raw kind 38383 event into an OrderEvent.
///
/// Standard Mostro order event tags:
/// - `d` → order id
/// - `amt` → amount_sats (integer satoshis)
/// - `f` → fiat_currency (normalized to uppercase)
/// - `fa` → fiat_amount
/// - `k` → order_side (case-insensitive → Buy/Sell/Unknown)
///
/// Other tags like `y` and `z` may be present but are not needed for v1 stats.
pub fn parse_order_event(ev: &NostrEvent) -> Result<OrderEvent, String> {
    let d_tag =
        find_tag_value(&ev.tags, "d").ok_or_else(|| format!("Missing d tag in event {}", ev.id))?;

    let amount_sats = find_tag_value(&ev.tags, "amt")
        .and_then(|v| v.parse::<u64>().ok().filter(|&n| n > 0))
        .ok_or_else(|| {
            format!(
                "Missing, non-numeric, or non-positive amt in order event {}",
                ev.id
            )
        })?;

    let fiat_currency = find_tag_value(&ev.tags, "f").map(|v| v.to_uppercase());
    let fiat_amount = find_tag_value(&ev.tags, "fa").and_then(|v| v.parse::<f64>().ok());

    let order_side = find_tag_value(&ev.tags, "k").map(|v| OrderSide::from_str(&v));

    Ok(OrderEvent {
        event_id: ev.id.clone(),
        d_tag,
        amount_sats,
        fiat_currency,
        fiat_amount,
        order_side,
    })
}

/// Find the first tag with the given key and return its first value.
fn find_tag_value(tags: &[Vec<String>], key: &str) -> Option<String> {
    tags.iter()
        .find(|t| t.first().map(|s| s.as_str()) == Some(key))
        .and_then(|t| t.get(1))
        .cloned()
}

/// Find the tag with the given key and return the value at the specified index.
fn find_tag_value_at_index(tags: &[Vec<String>], key: &str, index: usize) -> Option<String> {
    tags.iter()
        .find(|t| t.first().map(|s| s.as_str()) == Some(key))
        .and_then(|t| t.get(index))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NostrEvent;

    fn make_8383(tags: Vec<Vec<String>>) -> NostrEvent {
        NostrEvent {
            id: "abc123".to_string(),
            kind: 8383,
            pubkey: "aa".repeat(32),
            created_at: 1700000000,
            tags,
            content: String::new(),
        }
    }

    fn make_38383(tags: Vec<Vec<String>>) -> NostrEvent {
        NostrEvent {
            id: "def456".to_string(),
            kind: 38383,
            pubkey: "bb".repeat(32),
            created_at: 1700000000,
            tags,
            content: String::new(),
        }
    }

    #[test]
    fn parse_valid_8383() {
        let ev = make_8383(vec![
            vec!["order-id".into(), "order-1".into()],
            vec!["amount".into(), "500".into()],
        ]);
        let parsed = parse_dev_fee_event(&ev).unwrap();
        assert_eq!(parsed.order_id, "order-1");
        assert_eq!(parsed.fee_amount_sats, 500);
    }

    #[test]
    fn parse_8383_missing_order_id() {
        let ev = make_8383(vec![vec!["amount".into(), "100".into()]]);
        assert!(parse_dev_fee_event(&ev).is_err());
    }

    #[test]
    fn parse_8383_non_numeric_amount() {
        let ev = make_8383(vec![
            vec!["order-id".into(), "order-1".into()],
            vec!["amount".into(), "bad".into()],
        ]);
        assert!(parse_dev_fee_event(&ev).is_err());
    }

    #[test]
    fn parse_valid_38383() {
        let ev = make_38383(vec![
            vec!["d".into(), "order-1".into()],
            vec!["amt".into(), "1000000".into()],
            vec!["f".into(), "usd".into()],
            vec!["fa".into(), "50.0".into()],
            vec!["k".into(), "buy".into()],
            vec!["y".into(), "mostro".into()],
            vec!["z".into(), "dni".into()],
        ]);
        let parsed = parse_order_event(&ev).unwrap();
        assert_eq!(parsed.d_tag, "order-1");
        assert_eq!(parsed.amount_sats, 1000000);
        assert_eq!(parsed.fiat_currency, Some("USD".to_string()));
        assert_eq!(parsed.fiat_amount, Some(50.0));
        assert_eq!(parsed.order_side, Some(OrderSide::Buy));
    }

    #[test]
    fn parse_38383_missing_d() {
        let ev = make_38383(vec![vec!["amt".into(), "500".into()]]);
        assert!(parse_order_event(&ev).is_err());
    }

    #[test]
    fn parse_38383_zero_amt() {
        let ev = make_38383(vec![
            vec!["d".into(), "order-1".into()],
            vec!["amt".into(), "0".into()],
            vec!["f".into(), "usd".into()],
            vec!["fa".into(), "50.0".into()],
            vec!["k".into(), "buy".into()],
        ]);
        assert!(parse_order_event(&ev).is_err());
    }
}
