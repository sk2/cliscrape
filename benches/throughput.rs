use cliscrape::engine::types::Template;
use cliscrape::engine::types::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use tracing::Dispatch;
use tracing_subscriber::filter::LevelFilter;

fn benchmark_throughput(c: &mut Criterion) {
    let mut values = HashMap::new();
    values.insert(
        "Interface".to_string(),
        Value {
            name: "Interface".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: true,
            list: false,
            type_hint: None,
        },
    );
    values.insert(
        "Status".to_string(),
        Value {
            name: "Status".to_string(),
            regex: r#"\w+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
            type_hint: None,
        },
    );

    let mut states = HashMap::new();
    states.insert(
        "Start".to_string(),
        State {
            name: "Start".to_string(),
            rules: vec![Rule {
                regex: r#"Interface ${Interface} is ${Status}"#.to_string(),
                line_action: Action::Next,
                record_action: Action::Record,
                next_state: None,
            }],
        },
    );

    let ir = TemplateIR {
        values,
        states,
        macros: HashMap::new(),
    };

    let template = Template::from_ir(ir).unwrap();

    let mut input = String::new();
    for i in 0..100_000 {
        input.push_str(&format!(
            "Interface Gi0/{} is up
",
            i
        ));
        input.push_str(
            "  Some other line that doesn't match
",
        );
    }

    c.bench_function("parse 200k lines (baseline)", |b| {
        b.iter(|| {
            let _ = template.parse(black_box(&input)).unwrap();
        })
    });

    c.bench_function("parse 200k lines (tracing default)", |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::WARN)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            b.iter(|| {
                let _ = template.parse(black_box(&input)).unwrap();
            })
        });
    });

    c.bench_function("parse 200k lines (tracing off)", |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::OFF)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            b.iter(|| {
                let _ = template.parse(black_box(&input)).unwrap();
            })
        });
    });

    c.bench_function("parse 200k lines (tracing text)", |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            b.iter(|| {
                let _ = template.parse(black_box(&input)).unwrap();
            })
        });
    });

    c.bench_function("parse 200k lines (tracing json)", |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .json()
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || {
            b.iter(|| {
                let _ = template.parse(black_box(&input)).unwrap();
            })
        });
    });
}

criterion_group!(benches, benchmark_throughput);
criterion_main!(benches);
