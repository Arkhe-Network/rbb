pub mod adapter;
pub mod adapters;
pub mod identity_bridge;
pub mod pilots;

pub use adapter::{AcademicAdapter, AcademicRecord, AcademicRecordType, AdapterError};
pub use adapters::sigaa;
