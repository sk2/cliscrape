pub mod ios_prompt;

/// Preprocess raw CLI input that may include Cisco IOS-style prompts and command echoes.
///
/// Returns one or more per-command output blocks. When confidence is low, returns the original
/// input as a single block.
pub fn preprocess_ios_transcript(raw: &str) -> Vec<String> {
    let (blocks, _warnings) = preprocess_ios_transcript_with_warnings(raw);
    blocks
}

/// Preprocess raw CLI input, returning blocks plus any warnings (e.g., ANSI stripping).
pub fn preprocess_ios_transcript_with_warnings(raw: &str) -> (Vec<String>, Vec<String>) {
    let mut warnings = Vec::new();

    // Strip ANSI escape sequences before processing
    let clean_bytes = strip_ansi_escapes::strip(raw.as_bytes());
    let cleaned = String::from_utf8_lossy(&clean_bytes);

    // Check if ANSI stripping removed anything
    if cleaned.len() < raw.len() || cleaned.as_bytes() != raw.as_bytes() {
        warnings.push("ANSI escape sequences were stripped from input".to_string());
    }

    let blocks = ios_prompt::preprocess_ios_transcript(&cleaned);
    (blocks, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_escape_sequences_and_warns() {
        let input = "Host# show \x1b[31mcolored\x1b[0m output\nsome data";
        let (blocks, warnings) = preprocess_ios_transcript_with_warnings(input);

        // Should not contain ANSI codes
        let combined = blocks.join("\n");
        assert!(!combined.contains("\x1b["));
        assert!(!combined.contains("\x1b"));

        // Should have at least one warning
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("ANSI"));
    }

    #[test]
    fn no_warning_when_no_ansi_sequences() {
        let input = "Host# show plain output\nsome data";
        let (_blocks, warnings) = preprocess_ios_transcript_with_warnings(input);

        // No ANSI codes, so no warnings
        assert!(warnings.is_empty());
    }
}
