//! TEMPORARY (backend-f, plan 80 speed fixes): prints how long a step takes, wall and CPU of the
//! process, when `MANDI_PROFILE` is set. Removed before the handoff.
fn cpu_ms() -> f64 {
    let pid = std::process::id().to_string();
    let text = std::process::Command::new("ps")
        .args(["-o", "cputime=", "-p", &pid])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let mut seconds = 0.0_f64;
    for part in text.trim().split(':') {
        match part.trim().parse::<f64>() {
            Ok(n) => seconds = seconds * 60.0 + n,
            Err(_) => return -1.0,
        }
    }
    seconds * 1000.0
}

pub(crate) fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    if std::env::var_os("MANDI_PROFILE").is_none() {
        return f();
    }
    let (started, cpu) = (std::time::Instant::now(), cpu_ms());
    let result = f();
    eprintln!(
        "PROF {label}: wall {:?} cpu {:.0} ms",
        started.elapsed(),
        cpu_ms() - cpu
    );
    result
}
