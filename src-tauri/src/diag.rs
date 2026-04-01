//! Diagnostics: panic labeling and optional stage tracing.
//!
//! Set environment variable `YAEP_DIAG` to any value other than `0`, `false`, or empty
//! to print timestamped `YAEP_DIAG` lines to stderr (boot, thumbnail service, DWM).
//! Native crashes (e.g. access violation) do not run the panic hook; the last `YAEP_DIAG`
//! line before exit indicates the last completed stage.

use std::io::Write;
use std::sync::OnceLock;

pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let loc = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".into());
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("(non-string panic payload)");
        let _ = writeln!(
            std::io::stderr(),
            "YAEP_PANIC: {loc}: {payload} (RUST_BACKTRACE=1 for stack)"
        );
        default(info);
    }));
}

pub fn enabled() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| match std::env::var("YAEP_DIAG").ok().as_deref() {
        None | Some("") | Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => {
            false
        }
        _ => true,
    })
}

pub fn trace(module: &'static str, detail: &str) {
    if !enabled() {
        return;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = writeln!(std::io::stderr(), "YAEP_DIAG {ts} [{module}] {detail}");
    let _ = std::io::stderr().flush();
}
