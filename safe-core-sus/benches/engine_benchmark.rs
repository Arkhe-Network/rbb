use criterion::{black_box, criterion_group, criterion_main, Criterion};
use safe_core_sus::{EthicsRule, RuleEngine, Severity};

fn benchmark_evaluate(c: &mut Criterion) {
    let mut engine = RuleEngine::new();

    // Add multiple rules to simulate real-world load
    for i in 0..10 {
        engine
            .add_rule(EthicsRule {
                id: format!("BENCH-{}", i),
                pattern: ".*".into(),
                description: "Bench rule".into(),
                condition: "get_int(ctx, \"value\") > 10".into(),
                severity: Severity::Block,
                enabled: true,
            })
            .unwrap();
    }

    let ctx = serde_json::json!({
        "value": 20
    });

    c.bench_function("engine_evaluate", |b| {
        b.iter(|| black_box(engine.evaluate("action", &ctx).unwrap()))
    });
}

fn benchmark_add_rule(c: &mut Criterion) {
    let rule = EthicsRule {
        id: "ADD-BENCH".into(),
        pattern: ".*".into(),
        description: "Add rule bench".into(),
        condition: "get_int(ctx, \"value\") > 10".into(),
        severity: Severity::Block,
        enabled: true,
    };

    c.bench_function("engine_add_rule", |b| {
        b.iter(|| {
            let mut engine = RuleEngine::new();
            black_box(engine.add_rule(rule.clone()).unwrap())
        })
    });
}

criterion_group!(benches, benchmark_evaluate, benchmark_add_rule);
criterion_main!(benches);
