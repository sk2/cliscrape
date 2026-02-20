use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptLine<'a> {
    base: &'a str,
    cmd: Option<&'a str>,
}

fn prompt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Conservative IOS-ish prompt matcher:
        // - start-of-line
        // - hostname-ish token
        // - optional mode parens (e.g. (config-if))
        // - ends with # or >
        // - optional command echo after whitespace
        Regex::new(r"^(?P<base>[A-Za-z0-9_.:-]+)(?:\([^\r\n\)]*\))?[#>](?:[ \t]+(?P<cmd>.*))?$")
            .expect("prompt regex must compile")
    })
}

fn parse_prompt_line(line: &str) -> Option<PromptLine<'_>> {
    let caps = prompt_re().captures(line)?;
    let base = caps.name("base")?.as_str();
    let cmd = caps
        .name("cmd")
        .map(|m| m.as_str())
        .filter(|s| !s.trim().is_empty());
    Some(PromptLine { base, cmd })
}

/// Detect and segment a raw Cisco IOS-style transcript into per-command output blocks.
///
/// Behavior is conservative by design:
/// - Treat input as a transcript only when a prompt-like hostname base repeats at least twice, OR
///   when the very first line is a prompt+command echo.
/// - When confidence is low, returns the original input as a single block.
/// - Only strips prompt lines and command echo lines (no paging, banners, syslog noise, etc.).
pub fn preprocess_ios_transcript(raw: &str) -> Vec<String> {
    let mut base_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut first_line_is_echo = false;

    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if let Some(p) = parse_prompt_line(line) {
            *base_counts.entry(p.base).or_insert(0) += 1;
            if idx == 0 && p.cmd.is_some() {
                first_line_is_echo = true;
            }
        }
    }

    let has_stable_base = base_counts.values().any(|&n| n >= 2);
    if !has_stable_base && !first_line_is_echo {
        return vec![raw.to_string()];
    }

    let mut blocks: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut started_any = false;

    for line in raw.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(p) = parse_prompt_line(line) {
            match p.cmd {
                Some(_cmd) => {
                    // New command start. Finalize any previous block.
                    if !cur.is_empty() {
                        blocks.push(cur.join("\n"));
                        cur.clear();
                    }
                    started_any = true;
                    continue;
                }
                None => {
                    // Prompt-only. Finalize current block if present.
                    if !cur.is_empty() {
                        blocks.push(cur.join("\n"));
                        cur.clear();
                    }
                    continue;
                }
            }
        }

        // Normal line.
        if started_any {
            cur.push(line.to_string());
        }
    }

    if !cur.is_empty() {
        blocks.push(cur.join("\n"));
    }

    // If we never saw a prompt+command echo, we can't safely segment; fall back.
    if !started_any {
        return vec![raw.to_string()];
    }

    if blocks.is_empty() {
        vec![raw.to_string()]
    } else {
        blocks
    }
}
