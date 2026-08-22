use crate::types::StorageHandle;

#[derive(Debug, Clone, PartialEq)]
pub enum QmError {
    CoherenceTimeout,
    InvalidMode,
    InsufficientOpticalDepth,
    DarkCountLimit,
    HardwareFault,
    EnsembleNotFound,
}

#[derive(Debug, Clone)]
pub struct EitPulseSequence {
    pub control_rabi_mhz: f64,
    pub signal_detuning_mhz: f64,
    pub storage_ns: u64,
    pub mode_idx: u8,
}

#[derive(Debug, Clone)]
pub struct BeamsplitterConfig {
    pub reflectivity: f64,
    pub phase_rad: f64,
    pub detection_window_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhotonDetectionPattern {
    pub click_a: bool,
    pub click_b: bool,
    pub time_tag_a_ns: u64,
    pub time_tag_b_ns: u64,
    pub coincidence_window_ns: u64,
}

pub trait QuantumMemoryController {
    fn store(
        &mut self,
        mode: u8,
        pulse_params: &EitPulseSequence,
    ) -> Result<StorageHandle, QmError>;

    fn interfere_for_gbs(
        &mut self,
        handle_a: StorageHandle,
        handle_b: StorageHandle,
        bs_params: &BeamsplitterConfig,
    ) -> Result<PhotonDetectionPattern, QmError>;

    fn apply_qudit_cnot(
        &mut self,
        control: StorageHandle,
        target: StorageHandle,
        dim: u8,
    ) -> Result<(), QmError>;

    fn read_measurement(&self, handle: StorageHandle) -> Result<(u32, u32), QmError>;

    fn remaining_coherence_ns(&self, handle: StorageHandle) -> Result<u64, QmError>;
}
