use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cliscrape::FsmParser;

// Benchmark: cisco_ios_show_version template parsing performance
fn benchmark_cisco_ios_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/ios_show_version/ios_15_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_ios_show_version.yaml").unwrap();

    c.bench_function("cisco_ios_show_version", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

// Benchmark: cisco_ios_show_interfaces template parsing performance
fn benchmark_cisco_ios_show_interfaces(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/ios_show_interfaces/ios_15_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_ios_show_interfaces.yaml").unwrap();

    c.bench_function("cisco_ios_show_interfaces", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

// Benchmark: cisco_nxos_show_version template parsing performance
fn benchmark_cisco_nxos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/cisco/nxos_show_version/nxos_9_standard.txt");
    let parser = FsmParser::from_file("templates/cisco_nxos_show_version.yaml").unwrap();

    c.bench_function("cisco_nxos_show_version", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

// Benchmark: juniper_junos_show_version template parsing performance
fn benchmark_juniper_junos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/juniper/junos_show_version/junos_12_standard.txt");
    let parser = FsmParser::from_file("templates/juniper_junos_show_version.yaml").unwrap();

    c.bench_function("juniper_junos_show_version", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

// Benchmark: arista_eos_show_version template parsing performance
fn benchmark_arista_eos_show_version(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/arista/eos_show_version/eos_4_standard.txt");
    let parser = FsmParser::from_file("templates/arista_eos_show_version.yaml").unwrap();

    c.bench_function("arista_eos_show_version", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
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
