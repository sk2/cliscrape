use crate::ScraperError;
use std::collections::HashMap;

pub fn get_builtin_macros() -> HashMap<String, String> {
    let mut macros = HashMap::new();
    macros.insert(
        "ipv4".to_string(),
        r#"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#.to_string(),
    );
    // Common MAC formats seen in network CLI output and ntc-templates:
    // - aa:bb:cc:dd:ee:ff
    // - aabb.ccdd.eeff
    macros.insert(
        "mac_address".to_string(),
        r#"(?:[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2}){5}|[0-9a-fA-F]{4}\.[0-9a-fA-F]{4}\.[0-9a-fA-F]{4})"#.to_string(),
    );
    macros.insert("interface".to_string(), r#"\S+"#.to_string());
    macros.insert("word".to_string(), r#"\w+"#.to_string());
    macros.insert("eol".to_string(), r#"$"#.to_string());
    macros
}

pub fn expand_macros(
    regex: &str,
    local_overrides: &HashMap<String, String>,
) -> Result<String, ScraperError> {
    const MAX_DEPTH: usize = 10;

    // Local overrides shadow builtins.
    let mut all_macros = get_builtin_macros();
    for (name, value) in local_overrides {
        all_macros.insert(name.clone(), value.clone());
    }

    let mut cache = HashMap::<String, String>::new();
    let mut stack = Vec::<String>::new();
    expand_string(regex, &all_macros, &mut cache, &mut stack, 0, MAX_DEPTH)
}

fn expand_string(
    input: &str,
    macros: &HashMap<String, String>,
    cache: &mut HashMap<String, String>,
    stack: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) -> Result<String, ScraperError> {
    if depth > max_depth {
        return Err(ScraperError::Parse(format!(
            "Macro expansion exceeded max depth {max_depth}"
        )));
    }

    static TOKEN_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = TOKEN_RE.get_or_init(|| regex::Regex::new(r"\{\{([A-Za-z0-9_]+)\}\}").unwrap());

    let mut out = String::with_capacity(input.len());
    let mut last = 0;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).expect("macro token match");
        let name = cap.get(1).expect("macro token name").as_str();

        out.push_str(&input[last..m.start()]);

        if macros.contains_key(name) {
            let expanded = resolve_macro(name, macros, cache, stack, depth + 1, max_depth)?;
            out.push_str(&expanded);
        } else {
            // Unknown macro is an error during template load
            return Err(ScraperError::Parse(format!(
                "Unknown macro '{{{{{}}}}}' - macros must be defined before use",
                name
            )));
        }

        last = m.end();
    }

    out.push_str(&input[last..]);
    Ok(out)
}

fn resolve_macro(
    name: &str,
    macros: &HashMap<String, String>,
    cache: &mut HashMap<String, String>,
    stack: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) -> Result<String, ScraperError> {
    if let Some(v) = cache.get(name) {
        return Ok(v.clone());
    }

    if let Some(pos) = stack.iter().position(|s| s == name) {
        let mut chain = stack[pos..].to_vec();
        chain.push(name.to_string());
        return Err(ScraperError::Parse(format!(
            "Macro expansion cycle detected: {}",
            chain.join(" -> ")
        )));
    }

    let raw = macros.get(name).expect("macro exists").clone();
    stack.push(name.to_string());
    let expanded = expand_string(&raw, macros, cache, stack, depth, max_depth)?;
    stack.pop();

    cache.insert(name.to_string(), expanded.clone());
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        let local_overrides = HashMap::new();
        let regex = "Interface {{interface}} is {{word}}";
        let expanded = expand_macros(regex, &local_overrides).unwrap();
        assert_eq!(expanded, r#"Interface \S+ is \w+"#);
    }

    #[test]
    fn test_macro_shadowing() {
        let mut local_overrides = HashMap::new();
        local_overrides.insert("interface".to_string(), "Shadowed".to_string());
        let regex = "Interface {{interface}} is {{word}}";
        let expanded = expand_macros(regex, &local_overrides).unwrap();
        assert_eq!(expanded, r#"Interface Shadowed is \w+"#);
    }

    #[test]
    fn test_ipv4_expansion() {
        let local_overrides = HashMap::new();
        let regex = "IP {{ipv4}}";
        let expanded = expand_macros(regex, &local_overrides).unwrap();
        assert_eq!(expanded, r#"IP \d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#);
    }

    #[test]
    fn test_mac_address_expansion() {
        let local_overrides = HashMap::new();
        let regex = "MAC {{mac_address}}";
        let expanded = expand_macros(regex, &local_overrides).unwrap();
        assert_eq!(
            expanded,
            r#"MAC (?:[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2}){5}|[0-9a-fA-F]{4}\.[0-9a-fA-F]{4}\.[0-9a-fA-F]{4})"#
        );
    }

    #[test]
    fn test_nested_macro_expansion() {
        let mut local_overrides = HashMap::new();
        local_overrides.insert("inner".to_string(), "X".to_string());
        local_overrides.insert("outer".to_string(), "{{inner}}Y".to_string());

        let expanded = expand_macros("Start {{outer}} End", &local_overrides).unwrap();
        assert_eq!(expanded, "Start XY End");
    }

    #[test]
    fn test_macro_cycle_detection_errors() {
        let mut local_overrides = HashMap::new();
        local_overrides.insert("a".to_string(), "{{b}}".to_string());
        local_overrides.insert("b".to_string(), "{{a}}".to_string());

        let err = expand_macros("{{a}}", &local_overrides).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("cycle"), "{msg}");
        assert!(msg.contains("a"), "{msg}");
        assert!(msg.contains("b"), "{msg}");
    }
}
