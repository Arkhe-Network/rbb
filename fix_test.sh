sed -i 's/    std::sync::Arc::new(tokio::sync::Mutex::new(detector)),/    std::sync::Arc::new(tokio::sync::Mutex::new(detector)) as std::sync::Arc<tokio::sync::Mutex<dyn arkhe_vision::ObjectDetector>>,/g' crates/arkhe-vision/src/lib.rs
cargo test -p arkhe-vision
