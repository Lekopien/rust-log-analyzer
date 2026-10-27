use std::fs::File;
use std::io::{self, BufReader};
use std::process::ExitCode;

use logan::{analyze, report};

const USAGE: &str = "logan: analyze nginx/Apache access logs

USAGE:
    logan [OPTIONS] [FILE]...       read FILEs, or stdin when none are given

OPTIONS:
    -n, --top <N>        rows per section (default 10)
    -s, --status <CODE>  only count requests with this status
    -j, --json           print JSON instead of a text report
    -h, --help           show this help";

struct Args {
    top: usize,
    status: Option<u16>,
    json: bool,
    files: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        top: 10,
        status: None,
        json: false,
        files: vec![],
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => return Err(String::new()),
            "-j" | "--json" => args.json = true,
            "-n" | "--top" => {
                args.top = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--top needs a number")?
            }
            "-s" | "--status" => {
                args.status = Some(
                    it.next()
                        .and_then(|v| v.parse().ok())
                        .ok_or("--status needs a code")?,
                )
            }
            f if f.starts_with('-') && f != "-" => return Err(format!("unknown option {f}")),
            f => args.files.push(f.to_string()),
        }
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("error: {e}\n");
            }
            eprintln!("{USAGE}");
            return if e.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            };
        }
    };

    let mut total = logan::stats::Stats::default();
    let sources: Vec<String> = if args.files.is_empty() {
        vec!["-".into()]
    } else {
        args.files.clone()
    };
    for src in sources {
        let result = if src == "-" {
            analyze(io::stdin().lock(), args.status)
        } else {
            File::open(&src).and_then(|f| analyze(BufReader::new(f), args.status))
        };
        match result {
            Ok(s) => merge(&mut total, s),
            Err(e) => {
                eprintln!("error: {src}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let out = if args.json {
        report::json(&total, args.top)
    } else {
        report::text(&total, args.top)
    };
    println!("{out}");
    ExitCode::SUCCESS
}

fn merge(into: &mut logan::stats::Stats, s: logan::stats::Stats) {
    into.lines += s.lines;
    into.requests += s.requests;
    into.bytes += s.bytes;
    into.bots += s.bots;
    for (k, v) in s.status {
        *into.status.entry(k).or_default() += v;
    }
    for (k, v) in s.routes {
        *into.routes.entry(k).or_default() += v;
    }
    for (k, v) in s.ips {
        *into.ips.entry(k).or_default() += v;
    }
    for (k, v) in s.hours {
        *into.hours.entry(k).or_default() += v;
    }
    for (k, v) in s.not_found {
        *into.not_found.entry(k).or_default() += v;
    }
}
