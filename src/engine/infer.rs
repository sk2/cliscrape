use std::collections::HashSet;

pub struct InferenceEngine {
    samples: Vec<String>,
}

impl InferenceEngine {
    pub fn new(samples: Vec<String>) -> Self {
        Self { samples }
    }

    pub fn infer(&self) -> String {
        if self.samples.is_empty() {
            return "version: 1\nfields: {}\npatterns: []".to_string();
        }

        // 1. Identify common lines across samples
        let mut line_counts = std::collections::HashMap::new();
        let mut total_samples = 0;

        for sample in &self.samples {
            total_samples += 1;
            let lines: HashSet<_> = sample.lines().collect();
            for line in lines {
                *line_counts.entry(line).or_insert(0) += 1;
            }
        }

        // 2. Lines that appear in all samples are likely static "Anchors"
        // Lines that vary are candidate "Payloads"
        let mut pattern_lines = Vec::new();
        let representative = &self.samples[0];

        for line in representative.lines() {
            if line_counts.get(line).cloned().unwrap_or(0) == total_samples {
                // Static anchor
                pattern_lines.push(format!("  - regex: '^{}$'", escape_regex(line)));
            } else {
                // Potential variable line - try to find the varying part
                // For now, just generalize the whole line as a capture
                pattern_lines.push("  - regex: '^(?P<data>.*)$'\n    record: true".to_string());
            }
        }

        let mut yaml = String::from("version: 1\nfields:\n  data:\n    type: string\npatterns:\n");
        for p in pattern_lines {
            yaml.push_str(&p);
            yaml.push('\n');
        }

        yaml
    }
}

fn escape_regex(s: &str) -> String {
    regex::escape(s)
}
