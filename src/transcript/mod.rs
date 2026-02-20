pub mod ios_prompt;

/// Preprocess raw CLI input that may include Cisco IOS-style prompts and command echoes.
///
/// Returns one or more per-command output blocks. When confidence is low, returns the original
/// input as a single block.
pub fn preprocess_ios_transcript(raw: &str) -> Vec<String> {
    ios_prompt::preprocess_ios_transcript(raw)
}
