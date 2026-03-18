use ascii_canvas::core::ascii_export::{export_grid, ExportOptions};
use ascii_canvas::core::grid::Grid;
use std::time::Instant;

fn main() {
    println!("Running microbenchmarks...");

    benchmark_grid_mutation();
    benchmark_ascii_export();
}

fn benchmark_grid_mutation() {
    let mut grid = Grid::new(1000, 1000);
    let start = Instant::now();

    for y in 0..1000 {
        for x in 0..1000 {
            grid.set_char(x, y, 'X');
        }
    }

    let duration = start.elapsed();
    println!("Grid mutation (1M cells): {:?}", duration);
}

fn benchmark_ascii_export() {
    let mut grid = Grid::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            grid.set_char(x, y, '#');
        }
    }

    let options = ExportOptions::default();
    let start = Instant::now();

    for _ in 0..100 {
        let _ = export_grid(&grid, &options);
    }

    let duration = start.elapsed();
    println!(
        "ASCII export (10k cells x 100 iterations): {:?}",
        duration / 100
    );
}
