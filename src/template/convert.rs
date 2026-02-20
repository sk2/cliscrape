use crate::engine::types::{Action, FieldType, TemplateIR};
use crate::template::modern::{
    ActionDef, FieldDef, FieldTypeDef, LineActionDef, ModernTemplateDoc, RecordActionDef,
    StateRuleDef,
};
use std::collections::BTreeMap;

/// Best-effort conversion from the legacy `TemplateIR` (TextFSM lowering target)
/// into a strict-schema modern template document.
///
/// Notes:
/// - Output defaults all fields to explicit `string` typing unless the IR already has a type hint.
/// - Output uses explicit `states` (not `patterns`).
pub fn template_ir_to_modern_doc(ir: &TemplateIR) -> ModernTemplateDoc {
    let mut fields = BTreeMap::new();
    for (name, v) in &ir.values {
        let r#type = Some(match v.type_hint {
            Some(FieldType::Int) => FieldTypeDef::Int,
            Some(FieldType::String) | None => FieldTypeDef::String,
        });

        fields.insert(
            name.clone(),
            FieldDef {
                r#type,
                pattern: Some(v.regex.clone()),
                filldown: v.filldown,
                required: v.required,
                list: v.list,
            },
        );
    }

    let mut states = BTreeMap::new();
    for (state_name, st) in &ir.states {
        let rules = st
            .rules
            .iter()
            .map(|r| {
                let line = match r.line_action {
                    Action::Continue => Some(LineActionDef::Continue),
                    _ => None,
                };

                let record = match r.record_action {
                    Action::Record => Some(RecordActionDef::Record),
                    Action::Clear => Some(RecordActionDef::Clear),
                    _ => None,
                };

                let next = r.next_state.clone();

                let action = if line.is_none() && record.is_none() && next.is_none() {
                    None
                } else {
                    Some(ActionDef { line, record, next })
                };

                StateRuleDef {
                    regex: r.regex.clone(),
                    action,
                }
            })
            .collect();

        states.insert(state_name.clone(), rules);
    }

    ModernTemplateDoc {
        version: 1,
        macros: ir.macros.clone(),
        fields,
        states: Some(states),
        patterns: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Template;
    use crate::template::loader::TextFsmLoader;
    use crate::template::modern;

    #[test]
    fn convert_round_trips_through_modern_yaml_loader_and_parses_equivalently() {
        let textfsm = r#"Value IFACE (\\S+)
Value STATUS (up|down)

Start
  ^${IFACE} is ${STATUS} -> Record
"#;

        let ir = TextFsmLoader::parse_str(textfsm).unwrap();
        let legacy = Template::from_ir(ir.clone()).unwrap();

        let doc = template_ir_to_modern_doc(&ir);
        let yaml = modern::to_yaml_string(&doc).unwrap();
        let ir2 = modern::load_yaml_str(&yaml).unwrap();
        let modern_t = Template::from_ir(ir2).unwrap();

        let sample = "Eth1 is up";
        assert_eq!(
            legacy.parse(sample).unwrap(),
            modern_t.parse(sample).unwrap()
        );
    }

    #[test]
    fn convert_round_trips_through_modern_toml_loader_and_parses_equivalently() {
        let textfsm = r#"Value IFACE (\\S+)
Value STATUS (up|down)

Start
  ^${IFACE} is ${STATUS} -> Record
"#;

        let ir = TextFsmLoader::parse_str(textfsm).unwrap();
        let legacy = Template::from_ir(ir.clone()).unwrap();

        let doc = template_ir_to_modern_doc(&ir);
        let toml = modern::to_toml_string(&doc).unwrap();
        let ir2 = modern::load_toml_str(&toml).unwrap();
        let modern_t = Template::from_ir(ir2).unwrap();

        let sample = "Eth1 is down";
        assert_eq!(
            legacy.parse(sample).unwrap(),
            modern_t.parse(sample).unwrap()
        );
    }
}
