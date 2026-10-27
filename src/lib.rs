//! logan: analyze web server access logs.
pub mod parser;
pub mod report;
pub mod stats;

use std::io::BufRead;

/// Streams lines from a reader into `Stats`, one line in memory at a time.
pub fn analyze<R: BufRead>(reader: R, filter_status: Option<u16>) -> std::io::Result<stats::Stats> {
    let mut s = stats::Stats::default();
    for line in reader.lines() {
        let line = line?;
        s.lines += 1;
        match parser::parse_line(&line) {
            Some(e) if filter_status.is_none_or(|f| f == e.status) => s.add(&e),
            Some(_) => s.lines -= 1, // filtered out on purpose, not a parse failure
            None => {}
        }
    }
    Ok(s)
}
