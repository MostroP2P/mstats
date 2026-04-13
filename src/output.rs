use crate::models::ReportOutput;

/// Print human-readable report to stdout.
pub fn print_human_readable(report: &ReportOutput) {
    println!("=== Mostro Network Statistics ===\n");

    println!("Global:");
    println!(
        "  Orders:              {}",
        format_number(report.global.order_count)
    );
    println!(
        "  Total Dev Fees:      {} sats",
        format_number(report.global.total_fees_sats)
    );
    println!(
        "  Total Volume:        {} sats",
        format_number(report.global.total_volume_sats)
    );
    println!(
        "  Avg Order Size:      {} sats",
        format_number_f64(report.global.avg_order_size_sats)
    );

    if !report.global.fiat_volume_by_currency.is_empty() {
        println!("  Fiat Volume:");
        let mut currencies: Vec<_> = report.global.fiat_volume_by_currency.iter().collect();
        currencies.sort_by_key(|(k, _)| (*k).clone());
        for (currency, amount) in currencies {
            println!("    {}: {}", currency, format_fiat(*amount));
        }
    }

    if !report.global.volume_by_side.is_empty() {
        println!("  By Side:");
        let mut sides: Vec<_> = report.global.volume_by_side.iter().collect();
        sides.sort_by_key(|(k, _)| (*k).clone());
        for (side, volume) in sides {
            println!("    {}: {} sats", capitalize(side), format_number(*volume));
        }
    }

    if !report.nodes.is_empty() {
        println!("\n--- Per-Node Statistics ---\n");
        for ns in &report.nodes {
            println!("Node: {}{}", ns.node.pubkey, format_y(&ns.node.y_tag_value));
            println!("  Orders:              {}", format_number(ns.order_count));
            println!(
                "  Total Dev Fees:      {} sats",
                format_number(ns.total_fees_sats)
            );
            println!(
                "  Total Volume:        {} sats",
                format_number(ns.total_volume_sats)
            );
            println!(
                "  Avg Order Size:      {} sats",
                format_number_f64(ns.avg_order_size_sats)
            );

            if !ns.fiat_volume_by_currency.is_empty() {
                println!("  Fiat Volume:");
                let mut currencies: Vec<_> = ns.fiat_volume_by_currency.iter().collect();
                currencies.sort_by_key(|(k, _)| (*k).clone());
                for (currency, amount) in currencies {
                    println!("    {}: {}", currency, format_fiat(*amount));
                }
            }

            if !ns.volume_by_side.is_empty() {
                println!("  By Side:");
                let mut sides: Vec<_> = ns.volume_by_side.iter().collect();
                sides.sort_by_key(|(k, _)| (*k).clone());
                for (side, volume) in sides {
                    println!("    {}: {} sats", capitalize(side), format_number(*volume));
                }
            }
            println!();
        }
    }

    println!(
        "Summary: Processed {} kind 8383 events, joined with {} kind 38383 events.",
        report.data_quality.processed, report.data_quality.joined
    );
    println!("Data Quality:");
    println!("  Processed:  {}", report.data_quality.processed);
    println!("  Joined:     {}", report.data_quality.joined);
    println!("  Unmatched:  {}", report.data_quality.unmatched);
    println!("  Skipped:    {}", report.data_quality.skipped);
}

/// Print JSON output to stdout.
pub fn print_json(report: &ReportOutput) {
    let json = serde_json::to_string_pretty(report)
        .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize report: {}"}}"#, e));
    println!("{}", json);
}

fn format_number(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap_or_default()
        .join(",")
}

fn format_number_f64(n: f64) -> String {
    if n.fract() == 0.0 {
        format_number(n as u64)
    } else {
        format!("{:.2}", n)
    }
}

fn format_fiat(amount: f64) -> String {
    if amount.fract() == 0.0 {
        format!("{:.0}", amount)
    } else {
        format!("{:.2}", amount)
    }
}

fn format_y(y: &Option<String>) -> String {
    match y {
        Some(v) => format!("  (y: {})", v),
        None => String::new(),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
