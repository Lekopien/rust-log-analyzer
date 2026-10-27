//! Parser for the Combined Log Format used by nginx and Apache:
//! `ip - user [10/Oct/2026:13:55:36 +0000] "GET /path HTTP/1.1" 200 2326 "referer" "agent"`

/// One parsed request. Borrows from the input line, so parsing allocates nothing.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    pub ip: &'a str,
    pub time: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub status: u16,
    pub bytes: u64,
    pub referer: &'a str,
    pub agent: &'a str,
}

impl<'a> Entry<'a> {
    /// Hour bucket such as `10/Oct/2026:13`.
    pub fn hour(&self) -> &'a str {
        self.time.get(..14).unwrap_or(self.time)
    }

    /// Path without its query string.
    pub fn route(&self) -> &'a str {
        self.path.split('?').next().unwrap_or(self.path)
    }
}

/// Returns the text between `open` and `close` and the remainder after `close`.
fn between(s: &str, open: char, close: char) -> Option<(&str, &str)> {
    let start = s.find(open)? + open.len_utf8();
    let end = start + s[start..].find(close)?;
    Some((&s[start..end], &s[end + close.len_utf8()..]))
}

/// Parses one line, returning `None` for anything malformed.
pub fn parse_line(line: &str) -> Option<Entry<'_>> {
    let ip = line.split_whitespace().next()?;
    let (time, rest) = between(line, '[', ']')?;
    let (request, rest) = between(rest, '"', '"')?;
    let mut req = request.split_whitespace();
    let method = req.next()?;
    let path = req.next()?;

    let mut nums = rest.split_whitespace();
    let status = nums.next()?.parse().ok()?;
    let bytes = match nums.next()? {
        "-" => 0,
        n => n.parse().ok()?,
    };
    let (referer, rest) = between(rest, '"', '"').unwrap_or(("-", ""));
    let (agent, _) = between(rest, '"', '"').unwrap_or(("-", ""));
    Some(Entry {
        ip,
        time,
        method,
        path,
        status,
        bytes,
        referer,
        agent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"203.0.113.9 - - [27/Oct/2026:17:05:01 +0000] "GET /pricing?ref=ad HTTP/1.1" 200 5120 "https://google.com/" "Mozilla/5.0""#;

    #[test]
    fn parses_combined_format() {
        let e = parse_line(LINE).unwrap();
        assert_eq!(e.ip, "203.0.113.9");
        assert_eq!(e.method, "GET");
        assert_eq!(e.route(), "/pricing");
        assert_eq!(e.status, 200);
        assert_eq!(e.bytes, 5120);
        assert_eq!(e.hour(), "27/Oct/2026:17");
        assert_eq!(e.referer, "https://google.com/");
        assert_eq!(e.agent, "Mozilla/5.0");
    }

    #[test]
    fn accepts_dash_bytes_and_common_format() {
        let e = parse_line(r#"10.0.0.1 - - [27/Oct/2026:01:00:00 +0000] "HEAD / HTTP/1.1" 304 -"#)
            .unwrap();
        assert_eq!(e.bytes, 0);
        assert_eq!(e.agent, "-");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_line("").is_none());
        assert!(parse_line("not a log line").is_none());
        assert!(parse_line(r#"1.1.1.1 - - [x] "GET" 200 1"#).is_none());
        assert!(parse_line(r#"1.1.1.1 - - [x] "GET / HTTP/1.1" abc 1"#).is_none());
    }
}
