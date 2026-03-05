use cliscrape::FsmParser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tracing::Dispatch;
use tracing_subscriber::filter::LevelFilter;

fn bench_parse_variants(c: &mut Criterion, base: &str, parser: &FsmParser, input: &str) {
    let baseline = format!("{base} (baseline)");
    c.bench_function(&baseline, |b| b.iter(|| parser.parse(black_box(input))));

    let tracing_default = format!("{base} (tracing default)");
    c.bench_function(&tracing_default, |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::WARN)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || b.iter(|| parser.parse(black_box(input))))
    });

    let tracing_off = format!("{base} (tracing off)");
    c.bench_function(&tracing_off, |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::OFF)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || b.iter(|| parser.parse(black_box(input))))
    });

    let tracing_text = format!("{base} (tracing text)");
    c.bench_function(&tracing_text, |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || b.iter(|| parser.parse(black_box(input))))
    });

    let tracing_json = format!("{base} (tracing json)");
    c.bench_function(&tracing_json, |b| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::io::sink)
            .with_ansi(false)
            .json()
            .finish();
        let dispatch = Dispatch::new(subscriber);

        tracing::dispatcher::with_default(&dispatch, || b.iter(|| parser.parse(black_box(input))))
    });
}

// Benchmark: cisco_ios_show_version template parsing performance
fn benchmark_cisco_ios_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/ios_show_version/ios_15_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_ios_show_version.yaml").unwrap();

    bench_parse_variants(c, "cisco_ios_show_version", &parser, input);
}

// Benchmark: cisco_ios_show_interfaces template parsing performance
fn benchmark_cisco_ios_show_interfaces(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/ios_show_interfaces/ios_15_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_ios_show_interfaces.yaml").unwrap();

    bench_parse_variants(c, "cisco_ios_show_interfaces", &parser, input);
}

// Benchmark: cisco_nxos_show_version template parsing performance
fn benchmark_cisco_nxos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/nxos_show_version/nxos_9_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_nxos_show_version.yaml").unwrap();

    bench_parse_variants(c, "cisco_nxos_show_version", &parser, input);
}

// Benchmark: juniper_junos_show_version template parsing performance
fn benchmark_juniper_junos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/juniper/junos_show_version/junos_12_standard.txt");
    let parser = FsmParser::from_file("templates/juniper_junos_show_version.yaml").unwrap();

    bench_parse_variants(c, "juniper_junos_show_version", &parser, input);
}

// Benchmark: arista_eos_show_version template parsing performance
fn benchmark_arista_eos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/arista/eos_show_version/eos_4_standard.txt");
    let parser = FsmParser::from_file("templates/arista_eos_show_version.yaml").unwrap();

    bench_parse_variants(c, "arista_eos_show_version", &parser, input);
}

criterion_group!(
    benches,
    benchmark_cisco_ios_show_version,
    benchmark_cisco_ios_show_interfaces,
    benchmark_cisco_nxos_show_version,
    benchmark_juniper_junos_show_version,
    benchmark_arista_eos_show_version
);
criterion_main!(benches);
