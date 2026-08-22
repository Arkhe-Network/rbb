use ndarray::Array4;

fn main() {
    let _tensor = Array4::<f32>::zeros((1, 3, 640, 640));
    // let session = ort::Session::builder().unwrap().commit_from_file("dummy").unwrap();
    // let outputs = session.run(ort::inputs![tensor.view()]).unwrap();
}
