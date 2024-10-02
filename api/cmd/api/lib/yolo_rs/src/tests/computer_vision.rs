use crate::{YOLOv8, YOLOv8Config};
use std::env;

#[test]
fn test_face_detection_static_image() {
    let current_dir = env::current_dir().unwrap();
    println!("Diretório de trabalho atual: {:?}", current_dir);

    let image_source = "./assets/test-image-paulo.jpeg";

    let x = image::io::Reader::open(image_source)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    println!("Found image");

    let xs = vec![x];

    let mut model = YOLOv8::new(YOLOv8Config {
        model_path: String::from("./assets/YoloV8n-Face.onnx"),
        conf: 0.55,
        profile: true,
        plot: true,
        save_dir: Some("model-results".to_string()),
    })
    .unwrap();
    model.summary();

    let ys = model.run(&xs).unwrap();
    println!("{:?}", ys);
}
