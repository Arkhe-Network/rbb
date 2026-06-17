use tracing::{info, error, span, Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opentelemetry::trace::{SpanKind, Status, TraceContextExt};
use opentelemetry::metrics::{Meter, Counter, Histogram, MeterProvider};
use opentelemetry::global;
use std::sync::OnceLock;

use crate::testing::test_agent::{TestAgent, TestResult, TestType, TestContext};
use crate::testing::test_orchestrator::TestOrchestrator;
use crate::testing::test_attestation::TestAttestationExt;

/// Métricas globais para testes (inicializadas uma vez).
static METRICS: OnceLock<TestMetrics> = OnceLock::new();

struct TestMetrics {
    test_duration: Histogram<f64>,
    test_success: Counter<u64>,
    test_failure: Counter<u64>,
    test_total: Counter<u64>,
}

impl TestMetrics {
    fn init() -> &'static Self {
        METRICS.get_or_init(|| {
            let meter = global::meter("cathedral-arkhe-testing");
            Self {
                test_duration: meter
                    .f64_histogram("test.duration_seconds")
                    .with_description("Duration of test execution in seconds")
                    .build(),
                test_success: meter
                    .u64_counter("test.success_total")
                    .with_description("Total number of successful tests")
                    .build(),
                test_failure: meter
                    .u64_counter("test.failure_total")
                    .with_description("Total number of failed tests")
                    .build(),
                test_total: meter
                    .u64_counter("test.total")
                    .with_description("Total number of tests executed")
                    .build(),
            }
        })
    }
}

/// Extensão para adicionar tracing e métricas a TestAgent.
#[async_trait::async_trait]
pub trait TraceableTestAgent: TestAgent {
    async fn run_test_with_tracing_and_metrics(&self, context: &TestContext) -> Result<TestResult, String> {
        let span = span!(
            Level::INFO,
            "test.agent",
            test_name = %self.test_name(),
            test_type = ?self.test_type(),
            agent_id = %context.agent_id,
        );
        let _enter = span.enter();

        info!("🔄 Executando teste com tracing: {}", self.test_name());

        let start = std::time::Instant::now();
        let result = self.run_test(context).await;
        let duration = start.elapsed().as_secs_f64();

        // Atualizar métricas
        let metrics = TestMetrics::init();
        metrics.test_duration.record(duration, &[]);
        metrics.test_total.add(1, &[]);

        match &result {
            Ok(test_result) => {
                span.record("passed", &test_result.passed);
                span.record("duration_ms", &test_result.duration_ms);
                if test_result.passed {
                    metrics.test_success.add(1, &[]);
                    info!("✅ Teste concluído: {} (passou)", test_result.test_name);
                } else {
                    metrics.test_failure.add(1, &[]);
                    info!("❌ Teste concluído: {} (falhou)", test_result.test_name);
                }
            }
            Err(e) => {
                metrics.test_failure.add(1, &[]);
                span.record("error", &e);
                error!("❌ Teste falhou: {} - {}", self.test_name(), e);
            }
        }

        result
    }
}

/// Aplica automaticamente a todos os TestAgent.
impl<T: TestAgent + ?Sized> TraceableTestAgent for T {}

/// Atualiza o TestOrchestrator para usar tracing e métricas em cada teste.
impl TestOrchestrator {
    #[tracing::instrument(name = "test_orchestrator.run_all", skip(self))]
    pub async fn run_all_tests_with_tracing_and_metrics(&self) -> Vec<TestResult> {
        info!("🚀 Executando todos os testes com OpenTelemetry e métricas...");

        let context = crate::testing::test_agent::TestContext::new("orchestrator");

        let handles: Vec<_> = self.test_agents.iter()
            .map(|agent| {
                let ctx = context.clone();
                let agent_clone = agent.clone();
                tokio::spawn(async move {
                    agent_clone.run_test_with_tracing_and_metrics(&ctx).await
                })
            })
            .collect();

        let results = futures::future::join_all(handles).await;
        let mut test_results = Vec::new();

        for result in results {
            match result {
                Ok(Ok(test_result)) => {
                    // Persistir como TestAttestation
                    if let Err(e) = test_result.store_test_result_as_attestation(
                        self.signer.as_ref(),
                        self.store.as_ref(),
                    ).await {
                        error!("Falha ao persistir atestado de teste: {}", e);
                    }
                    test_results.push(test_result);
                }
                Ok(Err(e)) => error!("Erro no teste: {}", e),
                Err(e) => error!("Panic no teste: {}", e),
            }
        }

        self.generate_report(&test_results).await;
        info!("✅ Testes concluídos: {} resultados", test_results.len());
        test_results
    }

    // Add default run_all_tests wrapping the tracing/metrics one
    pub async fn run_all_tests(&self) -> Vec<TestResult> {
        self.run_all_tests_with_tracing_and_metrics().await
    }
}

// Chamar esta função na inicialização para configurar o meter provider.
pub fn init_test_metrics() {
    let _ = TestMetrics::init();
}
