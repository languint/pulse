use pulse_model::{LyricLine, Lyrics};

/// Parse LRC file contents into plain or synced lyrics.
#[must_use]
pub fn parse_lrc(content: &str) -> Option<Lyrics> {
    let mut synced = Vec::new();
    let mut plain_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (timestamps, text) = extract_timestamps(trimmed);
        if !timestamps.is_empty() {
            let text = text.trim();
            if text.is_empty() {
                continue;
            }
            for start_ms in timestamps {
                synced.push(LyricLine {
                    start_ms,
                    text: text.to_string(),
                });
            }
        } else if !trimmed.starts_with('[') {
            plain_lines.push(trimmed.to_string());
        }
    }

    if !synced.is_empty() {
        synced.sort_by_key(|line| line.start_ms);
        Some(Lyrics::Synced(synced))
    } else if !plain_lines.is_empty() {
        Some(Lyrics::Plain(plain_lines.join("\n")))
    } else {
        None
    }
}

fn extract_timestamps(line: &str) -> (Vec<u32>, &str) {
    let mut timestamps = Vec::new();
    let mut rest = line;

    while rest.starts_with('[') {
        let Some(end) = rest.find(']') else {
            break;
        };

        let tag = &rest[1..end];
        if let Some(ms) = parse_time_tag(tag) {
            timestamps.push(ms);
            rest = rest[end + 1..].trim_start();
        } else {
            break;
        }
    }

    (timestamps, rest)
}

fn parse_time_tag(tag: &str) -> Option<u32> {
    let (minutes, seconds, fraction) = parse_clock(tag)?;
    let fraction_ms = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<u32>().ok()? * 100,
        2 => fraction.parse::<u32>().ok()? * 10,
        3 => fraction.parse::<u32>().ok()?,
        _ => {
            let value = fraction.parse::<u32>().ok()?;
            value / 10
        }
    };

    Some(minutes * 60_000 + seconds * 1_000 + fraction_ms)
}

fn parse_clock(tag: &str) -> Option<(u32, u32, &str)> {
    let (left, fraction) = tag.split_once('.').unwrap_or((tag, ""));
    let mut parts = left.split(':');
    let minutes = parts.next()?.parse().ok()?;
    let seconds = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((minutes, seconds, fraction))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synced_lines() {
        let lrc = "[00:12.00]First line\n[00:15.50]Second line\n";
        let lyrics = parse_lrc(lrc).expect("lyrics");
        let lines = lyrics.synced_lines().expect("synced");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start_ms, 12_000);
        assert_eq!(lines[0].text, "First line");
        assert_eq!(lines[1].start_ms, 15_500);
    }

    #[test]
    fn parses_plain_lines_without_timestamps() {
        let lrc = "Line one\nLine two\n";
        let lyrics = parse_lrc(lrc).expect("lyrics");
        assert_eq!(lyrics.display_text(), "Line one\nLine two");
    }

    #[test]
    fn ignores_metadata_tags() {
        let lrc = "[ar:Artist]\n[00:01.00]Hello\n";
        let lyrics = parse_lrc(lrc).expect("lyrics");
        let lines = lyrics.synced_lines().expect("synced");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "Hello");
    }
}
