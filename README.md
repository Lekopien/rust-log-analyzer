# logan: Rust log analyzer

[![ci](https://github.com/Lekopien/rust-log-analyzer/actions/workflows/ci.yml/badge.svg)](https://github.com/Lekopien/rust-log-analyzer/actions/workflows/ci.yml)

**Project page: https://logs.lekopien.com**

A fast command-line analyzer for **nginx and Apache access logs**, written in Rust with **zero dependencies**. It streams files line by line (constant memory), parses without allocating, and prints a terminal dashboard or JSON for scripts.

## What it reports

- Total requests, traffic, unique IPs, 5xx error rate and bot share
- Top routes (query strings stripped), status codes and client IPs
- The most requested 404s, handy for spotting broken links and scanners like `/wp-login.php`
- Requests per hour with inline bar charts

## Install and run

```bash
git clone https://github.com/Lekopien/rust-log-analyzer
cd rust-log-analyzer
cargo run --release -- samples/access.log
cargo install --path .            # puts `logan` on your PATH
```

```text
$ logan samples/access.log -n 4
Requests        2000   skipped lines 1
Traffic      49.6 MB   unique IPs 40
5xx rate       3.45%   bot requests 820

Top routes
  /                                     600  ████████████████████
  /pricing                              253  ████████
  /services                             204  ███████
  /assets/app.js                        191  ██████

Status codes
  200                                  1669  ████████████████████
  404                                   144  ██
  304                                   118  █
  502                                    38  

Top IPs
  198.51.100.7                          681  ████████████████████
  203.0.113.33                           50  █
  203.0.113.8                            45  █
  203.0.113.15                           42  █
```

## Usage

```text
logan [OPTIONS] [FILE]...      read FILEs, or stdin when none are given

  -n, --top <N>        rows per section (default 10)
  -s, --status <CODE>  only count requests with this status
  -j, --json           print JSON instead of a text report
```

```bash
logan /var/log/nginx/access.log /var/log/nginx/access.log.1
zcat access.log.*.gz | logan --status 404 -n 20
logan access.log --json | jq '.topRoutes[0]'
```

## How it's built

| Module | Responsibility |
| --- | --- |
| `src/parser.rs` | Zero-copy parser: `Entry<'a>` borrows slices from the line |
| `src/stats.rs` | Aggregation with `HashMap`s, stable top-N ranking, byte formatting |
| `src/report.rs` | Text dashboard with bar charts, and hand-rolled JSON with proper escaping |
| `src/lib.rs` | `analyze()` streams any `BufRead` |
| `src/main.rs` | Argument parsing and merging stats across files |
| `tests/cli.rs` | End-to-end tests that run the compiled binary |

Malformed lines are skipped and counted rather than crashing the run. CI enforces `rustfmt`, `clippy -D warnings` and the test suite.

## License

MIT
