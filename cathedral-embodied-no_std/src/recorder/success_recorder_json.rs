use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRecord {
    pub round: u32,
    pub acceptance_rate: f32,
    pub memory_proof_used: bool,
}

pub struct SuccessRecorder {
    pub records: Vec<JsonRecord>,
    pub file_path: Option<String>,
}

impl SuccessRecorder {
    pub fn new(file_path: Option<&str>) -> Self {
        let records = if let Some(path) = file_path {
            if let Ok(data) = fs::read_to_string(path) {
                serde_json::from_str(&data).unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        Self {
            records,
            file_path: file_path.map(|s| s.to_string()),
        }
    }

    pub fn record_round(&mut self, round: u32, acceptance_rate: f32, memory_proof_used: bool) {
        self.records.push(JsonRecord {
            round,
            acceptance_rate,
            memory_proof_used,
        });
        self.flush();
    }

    pub fn average_acceptance_rate(&self, last_n: Option<usize>) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }
        let records_to_consider = match last_n {
            Some(n) => &self.records[self.records.len().saturating_sub(n)..],
            None => &self.records,
        };
        let sum: f32 = records_to_consider.iter().map(|r| r.acceptance_rate).sum();
        sum / records_to_consider.len() as f32
    }

    pub fn flush(&self) {
        if let Some(path) = &self.file_path {
            if let Ok(json) = serde_json::to_string(&self.records) {
                let _ = fs::write(path, json);
            }
        }
    }
}
