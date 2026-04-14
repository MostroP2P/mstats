use std::collections::HashMap;

use crate::models::{
    DataQualitySummary, GlobalStats, JoinedOrderRecord, NodeKey, NodeKeySerde, NodeStats,
    ReportOutput, UnjoinedRecord,
};

/// Aggregate joined records into global and per-node statistics.
pub fn aggregate(
    joined: Vec<JoinedOrderRecord>,
    unjoined: Vec<UnjoinedRecord>,
    skipped_count: u64,
) -> ReportOutput {
    let joined_count = joined.len() as u64;
    let unmatched_count = unjoined.len() as u64;

    // Global stats
    let mut global = GlobalStats {
        order_count: 0,
        total_fees_sats: 0,
        total_volume_sats: 0,
        avg_order_size_sats: 0.0,
        fiat_volume_by_currency: HashMap::new(),
        volume_by_side: HashMap::new(),
        source_event_ids: Vec::new(),
    };

    // Per-node stats
    let mut node_map: HashMap<String, (NodeKey, NodeStats)> = HashMap::new();

    for rec in &joined {
        // Global
        global.order_count += 1;
        global.total_fees_sats += rec.fee_event.fee_amount_sats;
        global.total_volume_sats += rec.order_event.amount_sats;
        global.source_event_ids.push(rec.fee_event.event_id.clone());
        global
            .source_event_ids
            .push(rec.order_event.event_id.clone());

        // Fiat volume
        if let (Some(currency), Some(amount)) =
            (&rec.order_event.fiat_currency, rec.order_event.fiat_amount)
        {
            *global
                .fiat_volume_by_currency
                .entry(currency.clone())
                .or_insert(0.0) += amount;
        }

        // Volume by side
        if let Some(side) = &rec.order_event.order_side {
            *global.volume_by_side.entry(side.to_string()).or_insert(0) +=
                rec.order_event.amount_sats;
        }

        // Per-node
        let entry = node_map.entry(rec.node_pubkey.clone()).or_insert_with(|| {
            let nk = NodeKey {
                pubkey: rec.node_pubkey.clone(),
                y_tag_value: rec.fee_event.y_tag_value.clone(),
            };
            let ns = NodeStats {
                node: NodeKeySerde::from(&nk),
                order_count: 0,
                total_fees_sats: 0,
                total_volume_sats: 0,
                avg_order_size_sats: 0.0,
                fiat_volume_by_currency: HashMap::new(),
                volume_by_side: HashMap::new(),
                source_event_ids: Vec::new(),
                _last_y_tag_created_at: 0,
            };
            (nk, ns)
        });

        entry.1.order_count += 1;
        entry.1.total_fees_sats += rec.fee_event.fee_amount_sats;
        entry.1.total_volume_sats += rec.order_event.amount_sats;
        entry
            .1
            .source_event_ids
            .push(rec.fee_event.event_id.clone());
        entry
            .1
            .source_event_ids
            .push(rec.order_event.event_id.clone());

        if let (Some(currency), Some(amount)) =
            (&rec.order_event.fiat_currency, rec.order_event.fiat_amount)
        {
            *entry
                .1
                .fiat_volume_by_currency
                .entry(currency.clone())
                .or_insert(0.0) += amount;
        }

        if let Some(side) = &rec.order_event.order_side {
            *entry.1.volume_by_side.entry(side.to_string()).or_insert(0) +=
                rec.order_event.amount_sats;
        }

        // Update y_tag_value: use value from most recently seen event (highest created_at)
        if let Some(yv) = &rec.fee_event.y_tag_value {
            let last_created = rec.fee_event.created_at;
            let should_update = match entry.0.y_tag_value.as_ref() {
                None => true,
                Some(_) => {
                    // Compare with the latest known created_at for this node's y_tag
                    rec.fee_event.created_at > entry.1._last_y_tag_created_at
                }
            };
            if should_update {
                entry.0.y_tag_value = Some(yv.clone());
                entry.1._last_y_tag_created_at = last_created;
                entry.1.node.y_tag_value = Some(yv.clone());
            }
        }
    }

    // Compute averages
    if global.order_count > 0 {
        global.avg_order_size_sats = global.total_volume_sats as f64 / global.order_count as f64;
    }

    for (_, ns) in node_map.values_mut() {
        if ns.order_count > 0 {
            ns.avg_order_size_sats = ns.total_volume_sats as f64 / ns.order_count as f64;
        }
    }

    // Sort nodes by order_count descending
    let mut nodes: Vec<NodeStats> = node_map.into_values().map(|(_, ns)| ns).collect();
    nodes.sort_by(|a, b| b.order_count.cmp(&a.order_count));

    // Data quality summary
    let processed = joined_count + unmatched_count + skipped_count;
    let data_quality = DataQualitySummary {
        processed,
        joined: joined_count,
        unmatched: unmatched_count,
        skipped: skipped_count,
    };

    ReportOutput {
        global,
        nodes,
        data_quality,
        unjoined,
        errors: Vec::new(),
        filter_summary: "No filters applied".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_joined(id: &str, pubkey: &str, fee: u64, sats: u64) -> JoinedOrderRecord {
        JoinedOrderRecord {
            fee_event: DevFeeEvent {
                event_id: format!("fee_{}", id),
                pubkey: pubkey.to_string(),
                created_at: 1700000000,
                order_id: id.to_string(),
                y_tag_value: None,
                fee_amount_sats: fee,
            },
            order_event: OrderEvent {
                event_id: format!("order_{}", id),
                d_tag: id.to_string(),
                amount_sats: sats,
                fiat_currency: Some("USD".to_string()),
                fiat_amount: Some(50.0),
                order_side: Some(OrderSide::Buy),
            },
            node_pubkey: pubkey.to_string(),
            order_id: id.to_string(),
        }
    }

    #[test]
    fn global_stats() {
        let joined = vec![
            make_joined("1", "aa", 100, 1000000),
            make_joined("2", "aa", 200, 2000000),
        ];
        let report = aggregate(joined, vec![], 0);
        assert_eq!(report.global.order_count, 2);
        assert_eq!(report.global.total_fees_sats, 300);
        assert_eq!(report.global.total_volume_sats, 3000000);
        assert!((report.global.avg_order_size_sats - 1500000.0).abs() < 0.01);
    }

    #[test]
    fn per_node_stats() {
        let joined = vec![
            make_joined("1", "aa", 100, 1000000),
            make_joined("2", "bb", 200, 2000000),
        ];
        let report = aggregate(joined, vec![], 0);
        assert_eq!(report.nodes.len(), 2);
    }

    #[test]
    fn quality_invariant() {
        let joined = vec![make_joined("1", "aa", 100, 1000000)];
        let unjoined = vec![UnjoinedRecord {
            event_id: "fx".to_string(),
            order_id: Some("ox".to_string()),
            pubkey: "aa".to_string(),
            y_tag_value: None,
            fee_amount_sats: Some(100),
            reason: UnjoinReason::OrderNotFound,
        }];
        let report = aggregate(joined, unjoined, 1);
        assert_eq!(report.data_quality.processed, 3); // 1+1+1
        assert_eq!(report.data_quality.joined, 1);
        assert_eq!(report.data_quality.unmatched, 1);
        assert_eq!(report.data_quality.skipped, 1);
    }

    #[test]
    fn source_event_ids() {
        let joined = vec![make_joined("1", "aa", 100, 1000000)];
        let report = aggregate(joined, vec![], 0);
        assert_eq!(report.global.source_event_ids.len(), 2);
    }
}
