//! Régénère `assets/icon.png` à partir de `assets/icon.svg` (R20).
//! Usage : `cargo run --example export_icon`

use resvg::{tiny_skia, usvg};

const SIZE: u32 = 256;

fn main() {
    let svg = std::fs::read_to_string("assets/icon.svg").expect("assets/icon.svg introuvable");
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("SVG invalide");

    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE).unwrap();
    let scale = SIZE as f32 / tree.size().width();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    pixmap
        .save_png("assets/icon.png")
        .expect("écriture de assets/icon.png impossible");
    println!("assets/icon.png écrit ({SIZE}×{SIZE})");
}
