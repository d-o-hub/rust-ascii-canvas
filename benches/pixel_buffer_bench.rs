use ascii_canvas::AsciiEditor;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("RUNNING PIXEL BUFFER RENDER BENCHMARKS (240x80 Grid)");
    println!("============================================================");

    benchmark_render_to_pixel_buffer_large_grid();
}

fn benchmark_render_to_pixel_buffer_large_grid() {
    // 1. Full Redraw on a large 240x80 grid
    let mut editor = AsciiEditor::new(240, 80);

    // Populate the grid with some content to simulate a complex diagram
    let mut cells_json = String::new();
    for y in 0..80 {
        for x in 0..240 {
            if (x + y) % 3 == 0 {
                if !cells_json.is_empty() {
                    cells_json.push(',');
                }
                cells_json.push_str(&format!(r#"{{"x":{},"y":{},"ch":"X"}}"#, x, y));
            }
        }
    }

    let doc_json = format!(
        r#"{{"format":"ascii-canvas","version":1,"canvas":{{"width":240,"height":80}},"active_layer":0,"layers":[{{"name":"Layer 1","visible":true,"cells":[{}]}}]}}"#,
        cells_json
    );

    assert!(editor.load_document(doc_json));

    // Force full redraw first
    editor.request_redraw();

    let start_full = Instant::now();
    let iterations = 500;
    for _ in 0..iterations {
        editor.request_redraw();
        editor.render_to_pixel_buffer();
    }
    let duration_full = start_full.elapsed();
    let avg_full = duration_full / iterations;
    println!(
        "Full Redraw (240x80 grid x {} iterations): {:?}",
        iterations, duration_full
    );
    println!("Average Full Redraw frame time: {:?}", avg_full);

    // 2. Partial/Dirty-Rect Redraw (simulate modifying 1 cell / small region)
    // Clear dirty state first
    editor.clear_dirty_state();

    // Measure rendering the partial update in a loop.
    // On each iteration, we clear the dirty state, mark a single cell dirty, and render it.
    let start_partial = Instant::now();
    for i in 0..iterations {
        editor.clear_dirty_state();
        editor.mark_cell_dirty_for_bench(80 + (i % 10) as i32, 40);
        editor.render_to_pixel_buffer();
    }
    let duration_partial = start_partial.elapsed();
    let avg_partial = duration_partial / iterations;
    println!(
        "Partial Redraw (1 cell x {} iterations): {:?}",
        iterations, duration_partial
    );
    println!("Average Partial Redraw frame time: {:?}", avg_partial);

    println!("------------------------------------------------------------");
    if avg_partial < avg_full {
        let speedup = avg_full.as_nanos() as f64 / avg_partial.as_nanos() as f64;
        println!(
            "Dirty-rect partial render is {:.2}x FASTER than full redraw!",
            speedup
        );
    } else {
        println!("Note: Frame timings are too small/noisy to compute speedup accurately.");
    }
    println!("============================================================");
}
