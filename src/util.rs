use std::fmt;
use std::time::Duration;

pub(crate) fn format_duration(f: &mut fmt::Formatter, duration: Duration) -> fmt::Result {
    let total_seconds = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let micros = nanos / 1_000;
    let millis = nanos / 1_000_000;
    let seconds = total_seconds % 60;
    let minutes = total_seconds / 60 % 60;
    let hours = total_seconds / 60 / 60;

    match (hours, minutes, seconds, millis, micros, nanos) {
        (0, 0, 0, 0, 0, n) => write!(f, "{}ns", n),
        (0, 0, 0, 0, u, _) => write!(f, "{}µs", u),
        (0, 0, 0, m, _, _) => write!(f, "{}ms", m),
        (0, 0, s, _, _, _) => write!(f, "{}s", s),
        (0, m, s, _, _, _) => write!(f, "{}:{:02}", m, s),
        (h, m, s, _, _, _) => write!(f, "{}:{:02}:{:02}", h, m, s),
    }
}
