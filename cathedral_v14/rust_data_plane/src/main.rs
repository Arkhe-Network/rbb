use std::fs;
use std::process;
use zmq::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    command: String,
    payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    status: String,
    data: serde_json::Value,
}

struct EpisodicMemory {
    memories: Vec<(Vec<f32>, serde_json::Value)>,
}

impl EpisodicMemory {
    fn new() -> Self { Self { memories: Vec::new() } }

    fn store(&mut self, emb: Vec<f32>, meta: serde_json::Value) {
        self.memories.push((emb, meta));
    }

    fn recall(&self, query: &Vec<f32>, k: usize) -> Vec<serde_json::Value> {
        if query.is_empty() || self.memories.is_empty() {
            return vec![];
        }
        let mut dists: Vec<(usize, f32)> = self.memories.iter().enumerate().map(|(i, (emb, _))| {
            let mut dot = 0.0;
            let len = query.len().min(emb.len());
            for j in 0..len { dot += query[j] * emb[j]; }
            (i, dot)
        }).collect();

        // Sort by dot product descending
        dists.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        dists.into_iter().take(k).map(|(i, _)| self.memories[i].1.clone()).collect()
    }
}

fn main() {
    let ctx = Context::new();
    let socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://127.0.0.1:5555").unwrap();
    println!("Rust Data Plane listening on tcp://127.0.0.1:5555");

    let mut memory = EpisodicMemory::new();
    let mut energy_budget = 20.0;

    loop {
        let msg = match socket.recv_string(0) {
            Ok(Ok(s)) => s,
            _ => continue,
        };

        let req: Request = match serde_json::from_str(&msg) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut resp = Response { status: "ok".to_string(), data: serde_json::Value::Null };

        match req.command.as_str() {
            "STORE" => {
                if let (Some(emb_val), Some(meta)) = (req.payload.get("embedding"), req.payload.get("metadata")) {
                    if let Ok(emb) = serde_json::from_value::<Vec<f32>>(emb_val.clone()) {
                        memory.store(emb, meta.clone());
                    }
                }
            },
            "RECALL" => {
                if let (Some(emb_val), Some(k_val)) = (req.payload.get("embedding"), req.payload.get("k")) {
                    if let (Ok(emb), Ok(k)) = (serde_json::from_value::<Vec<f32>>(emb_val.clone()), serde_json::from_value::<usize>(k_val.clone())) {
                        let results = memory.recall(&emb, k);
                        resp.data = serde_json::to_value(results).unwrap_or(serde_json::Value::Null);
                    }
                }
            },
            "UPDATE_ENERGY" => {
                if let Some(budget) = req.payload.get("budget").and_then(|b| b.as_f64()) {
                    energy_budget = budget as f32;
                    resp.data = serde_json::json!({"current_budget": energy_budget});
                }
            },
            "INTROSPECT" => {
                let pid: u32 = req.payload.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32).unwrap_or_else(|| process::id());
                let status = fs::read_to_string(format!("/proc/{}/status", pid)).unwrap_or_else(|_| "".to_string());

                // Self-Check Godeliano (simulado via Rust)
                let godelian_check = true;

                resp.data = serde_json::json!({
                    "status_len": status.len(),
                    "godelian_check": godelian_check,
                    "health": if godelian_check && !status.is_empty() { "nominal" } else { "degraded" },
                    "energy_budget": energy_budget
                });
            },
            _ => { resp.status = "error".to_string(); }
        }

        let _ = socket.send(&serde_json::to_string(&resp).unwrap_or("{}".to_string()), 0);
    }
}
