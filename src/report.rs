//! Renders a report as a text dashboard or JSON (hand-written, no serde needed).
use std::fmt::Write;

use crate::stats::{human_bytes, top, Stats};

fn bar(count: u64, max: u64, width: usize) -> String {
    let filled = if max == 0 {
        0
    } else {
        (count as f64 / max as f64 * width as f64).round() as usize
    };
    "█".repeat(filled)
}

pub fn text(s: &Stats, n: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Requests  {:>10}   skipped lines {}",
        s.requests,
        s.skipped()
    );
    let _ = writeln!(
        out,
        "Traffic   {:>10}   unique IPs {}",
        human_bytes(s.bytes),
        s.ips.len()
    );
    let _ = writeln!(
        out,
        "5xx rate  {:>9.2}%   bot requests {}",
        s.error_rate() * 100.0,
        s.bots
    );

    let mut section = |title: &str, rows: Vec<(String, u64)>| {
        let max = rows.first().map(|r| r.1).unwrap_or(0);
        let _ = writeln!(out, "\n{title}");
        for (k, c) in rows {
            let _ = writeln!(
                out,
                "  {:<32} {:>8}  {}",
                truncate(&k, 32),
                c,
                bar(c, max, 20)
            );
        }
    };
    section("Top routes", top(&s.routes, n));
    section(
        "Status codes",
        top(&s.status, n)
            .into_iter()
            .map(|(k, c)| (k.to_string(), c))
            .collect(),
    );
    section("Top IPs", top(&s.ips, n));
    section("Most requested 404s", top(&s.not_found, n));
    let mut hours: Vec<(String, u64)> = s.hours.iter().map(|(k, c)| (k.clone(), *c)).collect();
    hours.sort();
    section("Requests per hour", hours);
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max - 1).collect::<String>() + "…"
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_pairs(rows: Vec<(String, u64)>) -> String {
    let items: Vec<String> = rows
        .iter()
        .map(|(k, c)| format!("{{\"key\":{},\"count\":{c}}}", json_str(k)))
        .collect();
    format!("[{}]", items.join(","))
}

pub fn json(s: &Stats, n: usize) -> String {
    format!(
        "{{\"requests\":{},\"skipped\":{},\"bytes\":{},\"uniqueIps\":{},\"errorRate\":{:.4},\"bots\":{},\"topRoutes\":{},\"status\":{},\"topIps\":{},\"notFound\":{}}}",
        s.requests, s.skipped(), s.bytes, s.ips.len(), s.error_rate(), s.bots,
        json_pairs(top(&s.routes, n)),
        json_pairs(top(&s.status, n).into_iter().map(|(k, c)| (k.to_string(), c)).collect()),
        json_pairs(top(&s.ips, n)),
        json_pairs(top(&s.not_found, n)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_json() {
        assert_eq!(json_str("a\"b\\c\n"), r#""a\"b\\c\u000a""#);
    }

    #[test]
    fn draws_proportional_bars() {
        assert_eq!(bar(5, 10, 10).chars().count(), 5);
        assert_eq!(bar(0, 0, 10), "");
    }
}
