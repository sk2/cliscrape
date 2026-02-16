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

pub fn expand_macros(regex: &str, local_overrides: &HashMap<String, String>) -> String {
    let builtins = get_builtin_macros();
    let mut expanded = regex.to_string();

    // Collect all macro names to replace.
    // We use a simple approach as requested: replace {{name}} with values.
    // Since we want local_overrides to have priority, we'll check them first.

    // To implement simple replacement without recursion, we can just iterate over all possible macros.
    // This is efficient enough for small sets of macros.

    // We'll combine builtins and overrides, with overrides taking precedence.
    let mut all_macros = builtins;
    for (name, value) in local_overrides {
        all_macros.insert(name.clone(), value.clone());
    }

    for (name, value) in all_macros {
        let pattern = format!("{{{{{}}}}}", name);
        expanded = expanded.replace(&pattern, &value);
    }

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        let local_overrides = HashMap::new();
        let regex = "Interface {{interface}} is {{word}}";
        let expanded = expand_macros(regex, &local_overrides);
        assert_eq!(expanded, r#"Interface \S+ is \w+"#);
    }

    #[test]
    fn test_macro_shadowing() {
        let mut local_overrides = HashMap::new();
        local_overrides.insert("interface".to_string(), "Shadowed".to_string());
        let regex = "Interface {{interface}} is {{word}}";
        let expanded = expand_macros(regex, &local_overrides);
        assert_eq!(expanded, r#"Interface Shadowed is \w+"#);
    }

    #[test]
    fn test_ipv4_expansion() {
        let local_overrides = HashMap::new();
        let regex = "IP {{ipv4}}";
        let expanded = expand_macros(regex, &local_overrides);
        assert_eq!(expanded, r#"IP \d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#);
    }

    #[test]
    fn test_mac_address_expansion() {
        let local_overrides = HashMap::new();
        let regex = "MAC {{mac_address}}";
        let expanded = expand_macros(regex, &local_overrides);
        assert_eq!(
            expanded,
            r#"MAC (?:[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2}){5}|[0-9a-fA-F]{4}\.[0-9a-fA-F]{4}\.[0-9a-fA-F]{4})"#
        );
    }
}
