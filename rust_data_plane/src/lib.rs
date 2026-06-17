// src/lib.rs — Mínimo para compilar e gerar DLL / SO
#[no_mangle]
pub extern "C" fn arkhe_process_cycle(
    _input: *const f32,
    _len: usize,
) -> *mut f32 {
    // Stub: retorna vetor de zeros
    let result = vec![0.0f32; 4]; // action_dim = 4
    let mut boxed = result.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    ptr
}

#[no_mangle]
pub extern "C" fn arkhe_free(ptr: *mut f32, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
    }
}

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::exceptions::PyRuntimeError;
#[cfg(feature = "python")]
use std::sync::Arc;
#[cfg(feature = "python")]
use tokio::runtime::Runtime;

#[cfg(feature = "python")]
use orchestrator::testing::TestOrchestrator;
#[cfg(feature = "python")]
use orchestrator::testing::deps::{SubagentSpawner, AttestationManager, TrajectoryStore, AttestationSigner};
#[cfg(feature = "python")]
use orchestrator::testing::{
    IntegrityTestAgent, PerformanceTestAgent, ChaosTestAgent,
    SecurityTestAgent, ComplianceTestAgent, IntegrationTestAgent,
};

#[cfg(feature = "python")]
#[pyclass]
pub struct PyTestOrchestrator {
    inner: Arc<tokio::sync::Mutex<TestOrchestrator>>,
    rt: Runtime,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyTestOrchestrator {
    #[new]
    fn new(
        _spawner: PyObject,
        _attestation_manager: PyObject,
        _store: PyObject,
        _signer: PyObject,
    ) -> PyResult<Self> {
        // Mock extraction logic for the sake of the task.
        // We initialize it with default Dummy implementations from deps.
        use orchestrator::testing::deps::*;
        let signer = Arc::new(Ed25519Signer::new_random());
        let store = Arc::new(DummyTrajectoryStore::new());
        let att_manager = Arc::new(AttestationManager::new(Some(store.clone())));

        let parent_identity = Arc::new(tokio::sync::RwLock::new(IdentityAttestation::default()));
        let policy_engine = Arc::new(GeometricPolicyEngine::new());
        let sandbox = Arc::new(DummySandbox {});

        let spawner = Arc::new(SubagentSpawner::new(
            parent_identity,
            signer.clone() as Arc<dyn AttestationSigner + Send + Sync>,
            policy_engine,
            att_manager.clone(),
            store.clone(),
            50,
            sandbox,
            None,
        ));

        let orchestrator = TestOrchestrator::new(spawner, att_manager, store, signer);

        let rt = Runtime::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(tokio::sync::Mutex::new(orchestrator)),
            rt,
        })
    }

    fn register_integrity_test(&self, max_samples: usize) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let att_manager = orch.attestation_manager.clone();
            let store = orch.store.clone();
            let signer = orch.signer.clone();
            let agent = IntegrityTestAgent::new(att_manager, store, signer, max_samples);
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn register_performance_test(&self, concurrency: usize) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let spawner = orch.spawner.clone();
            let signer = orch.signer.clone();
            let agent = PerformanceTestAgent::new(spawner, signer, concurrency);
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn register_chaos_test(&self, failure_rate: f64, kill_percentage: f32) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let spawner = orch.spawner.clone();
            let agent = ChaosTestAgent::new(spawner, failure_rate, kill_percentage);
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn register_security_test(&self) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let agent = SecurityTestAgent::new();
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn register_compliance_test(&self, required_policies: Vec<String>) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let policy_engine = Arc::new(orchestrator::testing::deps::GeometricPolicyEngine::new());
            let att_manager = orch.attestation_manager.clone();
            let store = orch.store.clone();
            let signer = orch.signer.clone();
            let agent = ComplianceTestAgent::new(policy_engine, att_manager, store, signer, required_policies);
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn register_integration_test(&self, test_count: usize) -> PyResult<()> {
        let inner = self.inner.clone();
        self.rt.block_on(async move {
            let mut orch = inner.lock().await;
            let spawner = orch.spawner.clone();
            let att_manager = orch.attestation_manager.clone();
            let store = orch.store.clone();
            let signer = orch.signer.clone();
            let agent = IntegrationTestAgent::new(spawner, att_manager, store, signer, test_count);
            orch.register_test_agent(Arc::new(agent)).await;
        });
        Ok(())
    }

    fn run_all_tests(&self) -> PyResult<String> {
        let inner = self.inner.clone();
        let results = self.rt.block_on(async move {
            let orch = inner.lock().await;
            orch.run_all_tests().await
        });
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| PyRuntimeError::new_err(format!("Serialization error: {}", e)))?;
        Ok(json)
    }

    fn stats(&self) -> PyResult<String> {
        let inner = self.inner.clone();
        let stats = self.rt.block_on(async move {
            let orch = inner.lock().await;
            orch.stats().await
        });
        let json = serde_json::to_string_pretty(&stats)
            .map_err(|e| PyRuntimeError::new_err(format!("Serialization error: {}", e)))?;
        Ok(json)
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn cathedral_arkhe(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTestOrchestrator>()?;
    Ok(())
}
