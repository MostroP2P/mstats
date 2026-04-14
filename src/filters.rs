use crate::models::JoinedOrderRecord;

/// Filter predicate container. All active filters are AND-composed.
pub fn apply_filters(
    records: Vec<JoinedOrderRecord>,
    from_ts: Option<u64>,
    to_ts: Option<u64>,
    node_pubkey: Option<&str>,
    currency: Option<&str>,
    side: Option<&str>,
) -> Vec<JoinedOrderRecord> {
    records
        .into_iter()
        .filter(|rec| {
            // Date range filter (on kind 8383 created_at)
            if let Some(from) = from_ts {
                if rec.fee_event.created_at < from {
                    return false;
                }
            }
            if let Some(to) = to_ts {
                if rec.fee_event.created_at > to {
                    return false;
                }
            }

            // Node pubkey filter
            if let Some(pk) = node_pubkey {
                if rec.node_pubkey != pk {
                    return false;
                }
            }

            // Fiat currency filter (case-insensitive, both sides uppercased)
            if let Some(cur) = currency {
                let rec_cur = rec.order_event.fiat_currency.as_deref().unwrap_or("");
                if rec_cur.to_uppercase() != cur.to_uppercase() {
                    return false;
                }
            }

            // Order side filter
            if let Some(s) = side {
                let rec_side = rec
                    .order_event
                    .order_side
                    .as_ref()
                    .map(|os| os.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                if rec_side != s {
                    return false;
                }
            }

            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_joined(
        pubkey: &str,
        created_at: u64,
        fiat: Option<&str>,
        side: OrderSide,
    ) -> JoinedOrderRecord {
        JoinedOrderRecord {
            fee_event: DevFeeEvent {
                event_id: "f1".to_string(),
                pubkey: pubkey.to_string(),
                created_at,
                order_id: "o1".to_string(),
                y_tag_value: None,
                fee_amount_sats: 100,
            },
            order_event: OrderEvent {
                event_id: "oe1".to_string(),
                d_tag: "o1".to_string(),
                amount_sats: 1000000,
                fiat_currency: fiat.map(|s| s.to_string()),
                fiat_amount: Some(50.0),
                order_side: Some(side),
            },
            node_pubkey: pubkey.to_string(),
            order_id: "o1".to_string(),
        }
    }

    #[test]
    fn date_filter() {
        let recs = vec![
            make_joined("aa", 1700000000, Some("USD"), OrderSide::Buy),
            make_joined("bb", 1800000000, Some("EUR"), OrderSide::Sell),
        ];
        let filtered = apply_filters(recs, Some(1750000000), None, None, None, None);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn currency_case_insensitive() {
        let recs = vec![
            make_joined("aa", 1700000000, Some("usd"), OrderSide::Buy),
            make_joined("bb", 1700000000, Some("EUR"), OrderSide::Sell),
        ];
        let filtered = apply_filters(recs, None, None, None, Some("USD"), None);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn side_filter() {
        let recs = vec![
            make_joined("aa", 1700000000, Some("USD"), OrderSide::Buy),
            make_joined("bb", 1700000000, Some("USD"), OrderSide::Sell),
        ];
        let filtered = apply_filters(recs, None, None, None, None, Some("buy"));
        assert_eq!(filtered.len(), 1);
    }
}
