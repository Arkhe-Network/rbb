sed -i 's/            let outputs = self/            let outputs = self.session.run(ort::inputs![tensor_value].unwrap()).map_err(|e| VisionError::Inference(e.to_string()))?;/' crates/arkhe-vision/src/lib.rs
sed -i '/                .session/d' crates/arkhe-vision/src/lib.rs
sed -i '/                .run(ort::inputs!\[tensor_value\])/d' crates/arkhe-vision/src/lib.rs
sed -i '/                .map_err(|e| VisionError::Inference(e.to_string()))?;/d' crates/arkhe-vision/src/lib.rs
