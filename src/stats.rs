//! Aggregates parsed entries into a report.
use std::collections::HashMap;

use crate::parser::Entry;

#[derive(Default)]
pub struct Stats {
    pub lines: u64,
    pub requests: u64,
    pub bytes: u64,
    pub status: HashMap<u16, u64>,
    pub routes: HashMap<String, u64>,
    pub ips: HashMap<String, u64>,
    pub hours: HashMap<String, u64>,
    pub not_found: HashMap<String, u64>,
    pub bots: u64,
}

const BOT_HINTS: [&str; 5] = ["bot", "crawler", "spider", "curl", "python-requests"];

impl Stats {
    pub fn add(&mut self, e: &Entry) {
        self.requests += 1;
        self.bytes += e.bytes;
        *self.status.entry(e.status).or_default() += 1;
        *self.routes.entry(e.route().to_owned()).or_default() += 1;
        *self.ips.entry(e.ip.to_owned()).or_default() += 1;
        *self.hours.entry(e.hour().to_owned()).or_default() += 1;
        if e.status == 404 {
            *self.not_found.entry(e.route().to_owned()).or_default() += 1;
        }
        let agent = e.agent.to_ascii_lowercase();
        if BOT_HINTS.iter().any(|b| agent.contains(b)) {
            self.bots += 1;
        }
    }

    pub fn skipped(&self) -> u64 {
        self.lines - self.requests
    }

    /// Share of responses with a 5xx status.
    pub fn error_rate(&self) -> f64 {
        let errors: u64 = self
            .status
            .iter()
            .filter(|(s, _)| **s >= 500)
            .map(|(_, n)| n)
            .sum();
        if self.requests == 0 {
            0.0
        } else {
            errors as f64 / self.requests as f64
        }
    }
}

/// The `n` largest entries, ties broken alphabetically so output is stable.
pub fn top<K: Ord + Clone>(map: &HashMap<K, u64>, n: usize) -> Vec<(K, u64)> {
    let mut v: Vec<(K, u64)> = map.iter().map(|(k, c)| (k.clone(), *c)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

pub fn human_bytes(b: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = b as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{b} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_line;

    #[test]
    fn aggregates_and_ranks() {
        let mut s = Stats::default();
        for line in [
            r#"1.1.1.1 - - [27/Oct/2026:10:00:00 +0000] "GET /a HTTP/1.1" 200 100 "-" "Mozilla""#,
            r#"1.1.1.1 - - [27/Oct/2026:10:30:00 +0000] "GET /a?x=1 HTTP/1.1" 200 100 "-" "Googlebot""#,
            r#"2.2.2.2 - - [27/Oct/2026:11:00:00 +0000] "GET /missing HTTP/1.1" 404 0 "-" "Mozilla""#,
            r#"3.3.3.3 - - [27/Oct/2026:11:00:00 +0000] "POST /api HTTP/1.1" 502 0 "-" "curl/8""#,
        ] {
            s.lines += 1;
            s.add(&parse_line(line).unwrap());
        }
        assert_eq!(s.requests, 4);
        assert_eq!(s.bytes, 200);
        assert_eq!(top(&s.routes, 1), vec![("/a".to_string(), 2)]);
        assert_eq!(top(&s.ips, 1)[0].0, "1.1.1.1");
        assert_eq!(s.not_found["/missing"], 1);
        assert_eq!(s.bots, 2);
        assert!((s.error_rate() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn formats_bytes() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1536), "1.5 KB");
        assert_eq!(human_bytes(5 * 1024 * 1024), "5.0 MB");
    }
}
