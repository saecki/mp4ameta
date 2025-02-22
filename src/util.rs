use std::fmt;
use std::time::Duration;

use crate::Chapter;

pub(crate) fn format_duration(f: &mut fmt::Formatter<'_>, duration: Duration) -> fmt::Result {
    let total_seconds = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let micros = nanos / 1_000;
    let millis = nanos / 1_000_000;
    let seconds = total_seconds % 60;
    let minutes = total_seconds / 60 % 60;
    let hours = total_seconds / 60 / 60;

    match (hours, minutes, seconds, millis, micros, nanos) {
        (0, 0, 0, 0, 0, n) => write!(f, "{n}ns"),
        (0, 0, 0, 0, u, _) => write!(f, "{u}µs"),
        (0, 0, 0, m, _, _) => write!(f, "{m}ms"),
        (0, 0, s, _, _, _) => write!(f, "{s}s"),
        (0, m, s, _, _, _) => write!(f, "{m}:{s:02}"),
        (h, m, s, _, _, _) => write!(f, "{h}:{m:02}:{s:02}"),
    }
}

pub(crate) fn format_chapters(
    f: &mut fmt::Formatter<'_>,
    chapters: &[Chapter],
    duration: Duration,
) -> fmt::Result {
    for (i, c) in chapters.iter().enumerate() {
        writeln!(f, "    {}", c.title)?;
        if c.start == Duration::ZERO {
            f.write_str("      start: 0:00")?;
        } else {
            f.write_str("      start: ")?;
            format_duration(f, c.start)?;
        }

        let end = chapters.get(i + 1).map(|c| c.start).unwrap_or(duration);
        let duration = end.saturating_sub(c.start);
        f.write_str(", duration: ")?;
        format_duration(f, duration)?;
        writeln!(f)?;
    }
    Ok(())
}
