use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cliscrape::engine::types::*;
use cliscrape::engine::types::Template;
use std::collections::HashMap;

fn benchmark_throughput(c: &mut Criterion) {
    let mut values = HashMap::new();
    values.insert("Interface".to_string(), Value {
        name: "Interface".to_string(),
        regex: r#"\S+"#.to_string(),
        filldown: false,
        required: true,
        list: false,
        type_hint: None,
    });
    values.insert("Status".to_string(), Value {
        name: "Status".to_string(),
        regex: r#"\w+"#.to_string(),
        filldown: false,
        required: false,
        list: false,
        type_hint: None,
    });

    let mut states = HashMap::new();
    states.insert("Start".to_string(), State {
        name: "Start".to_string(),
        rules: vec![
            Rule {
                regex: r#"Interface ${Interface} is ${Status}"#.to_string(),
                line_action: Action::Next,
                record_action: Action::Record,
                next_state: None,
            },
        ],
    });

    let ir = TemplateIR {
        values,
        states,
        macros: HashMap::new(),
    };

    let template = Template::from_ir(ir).unwrap();
    
    let mut input = String::new();
    for i in 0..100_000 {
        input.push_str(&format!("Interface Gi0/{} is up
", i));
        input.push_str("  Some other line that doesn't match
");
    }

    c.bench_function("parse 200k lines", |b| {
        b.iter(|| {
            let _ = template.parse(black_box(&input)).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_throughput);
criterion_main!(benches);
