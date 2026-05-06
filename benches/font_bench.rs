use ascii_canvas::render::FontAtlas;
use std::time::Instant;

fn main() {
    println!("Running FontAtlas benchmarks...");

    benchmark_font_atlas_new();
    benchmark_font_atlas_render();
}

fn benchmark_font_atlas_new() {
    let start = Instant::now();

    for _ in 0..1000 {
        let _ = FontAtlas::new();
    }

    let duration = start.elapsed();
    println!("FontAtlas::new() (1000 iterations): {:?}", duration);
}

fn benchmark_font_atlas_render() {
    let atlas = FontAtlas::new();
    let mut buffer = vec![0u8; 800 * 600 * 4];
    let start = Instant::now();

    for _ in 0..100 {
        for y in (0..600).step_by(20) {
            for x in (0..800).step_by(8) {
                atlas.render_glyph(&mut buffer, 800, x, y, 'A', [255, 255, 255, 255]);
            }
        }
    }

    let duration = start.elapsed();
    println!(
        "FontAtlas::render_glyph (800x600 screen x 100 iterations): {:?}",
        duration
    );
}
