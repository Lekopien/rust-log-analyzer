use std::process::Command;

fn logan(args: &[&str]) -> (String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_logan"))
        .args(args)
        .output()
        .unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.success())
}

#[test]
fn text_report_on_sample() {
    let (out, ok) = logan(&["samples/access.log", "-n", "3"]);
    assert!(ok);
    assert!(out.contains("Requests        2000   skipped lines 1"));
    assert!(out.contains("Top routes"));
    assert!(out.contains("/wp-login.php"));
}

#[test]
fn json_and_status_filter() {
    let (out, ok) = logan(&["samples/access.log", "--json", "--status", "404"]);
    assert!(ok);
    assert!(out.starts_with("{\"requests\":"));
    assert!(out.contains("\"key\":\"404\""));
    assert!(!out.contains("\"key\":\"200\""));
}

#[test]
fn bad_flag_fails() {
    let (_, ok) = logan(&["--wat"]);
    assert!(!ok);
}
