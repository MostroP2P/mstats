use std::collections::HashMap;

use crate::models::{DevFeeEvent, JoinedOrderRecord, OrderEvent, UnjoinReason, UnjoinedRecord};

/// Join DevFeeEvents with OrderEvents by matching order_id to d_tag.
///
/// Returns (joined_records, unjoined_records).
pub fn join_events(
    fee_events: &[DevFeeEvent],
    order_events: &[OrderEvent],
) -> (Vec<JoinedOrderRecord>, Vec<UnjoinedRecord>) {
    // Index order events by d_tag for O(1) lookup
    let order_map: HashMap<&str, &OrderEvent> =
        order_events.iter().map(|o| (o.d_tag.as_str(), o)).collect();

    let mut joined = Vec::new();
    let mut unjoined = Vec::new();

    for fee in fee_events {
        if let Some(order) = order_map.get(fee.order_id.as_str()) {
            joined.push(JoinedOrderRecord {
                fee_event: fee.clone(),
                order_event: (*order).clone(),
                node_pubkey: fee.pubkey.clone(),
                order_id: fee.order_id.clone(),
            });
        } else {
            unjoined.push(UnjoinedRecord {
                event_id: fee.event_id.clone(),
                order_id: Some(fee.order_id.clone()),
                pubkey: fee.pubkey.clone(),
                y_tag_value: fee.y_tag_value.clone(),
                fee_amount_sats: Some(fee.fee_amount_sats),
                reason: UnjoinReason::OrderNotFound,
            });
        }
    }

    (joined, unjoined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DevFeeEvent, OrderEvent, OrderSide};

    fn make_fee(id: &str, order_id: &str, amount: u64) -> DevFeeEvent {
        DevFeeEvent {
            event_id: id.to_string(),
            pubkey: "aa".repeat(32),
            created_at: 1700000000,
            order_id: order_id.to_string(),
            y_tag_value: None,
            fee_amount_sats: amount,
        }
    }

    fn make_order(d: &str, sats: u64) -> OrderEvent {
        OrderEvent {
            event_id: "oe1".to_string(),
            d_tag: d.to_string(),
            amount_sats: sats,
            fiat_currency: Some("USD".to_string()),
            fiat_amount: Some(50.0),
            order_side: Some(OrderSide::Buy),
        }
    }

    #[test]
    fn join_matching() {
        let fees = vec![make_fee("f1", "o1", 100)];
        let orders = vec![make_order("o1", 1000000)];
        let (joined, unjoined) = join_events(&fees, &orders);
        assert_eq!(joined.len(), 1);
        assert_eq!(unjoined.len(), 0);
    }

    #[test]
    fn join_no_match() {
        let fees = vec![make_fee("f1", "o999", 100)];
        let orders = vec![make_order("o1", 1000000)];
        let (joined, unjoined) = join_events(&fees, &orders);
        assert_eq!(joined.len(), 0);
        assert_eq!(unjoined.len(), 1);
        assert_eq!(unjoined[0].pubkey, "aa".repeat(32));
        assert_eq!(unjoined[0].fee_amount_sats, Some(100));
        assert!(matches!(unjoined[0].reason, UnjoinReason::OrderNotFound));
    }
}
