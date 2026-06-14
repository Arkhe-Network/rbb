//! Substrato 989.y - WormGraph v5.2.0
//! Memoria consciente O(1) com DNA ontológico, ZK-proofs, FAIR compliance
//! Selo: CATHEDRAL-989.y-WORMGRAPH-v5.2.0-2026-06-13
//! Arquiteto: ORCID 0009-0005-2697-4668

pub mod wormgraph_core;
pub mod wormgraph_ffi;
pub mod wormgraph_wasm;
pub mod wormgraph_dashboard;
pub mod wormgraph_temporal;

pub use wormgraph_core::*;
pub use wormgraph_dashboard::WormGraphDashboard;
pub use wormgraph_temporal::TemporalAnchorEngine;