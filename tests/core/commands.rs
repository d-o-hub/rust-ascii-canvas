//! Command tests.

use ascii_canvas::core::cell::Cell;
use ascii_canvas::core::commands::{
    ClearCellCommand, ClearGridCommand, Command, CompositeCommand, DrawCommand, SetCellCommand,
};
use ascii_canvas::core::grid::Grid;
use ascii_canvas::core::tools::DrawOp;

#[test]
fn test_set_cell_command() {
    let mut grid = Grid::new(10, 10);
    let mut cmd = SetCellCommand::new(5, 5, Cell::new('X'));

    cmd.apply(&mut grid);
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');

    cmd.undo(&mut grid);
    assert!(grid.get(5, 5).unwrap().is_empty());
}

#[test]
fn test_clear_cell_command() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(5, 5, 'X');

    let mut cmd = ClearCellCommand::new(5, 5);
    cmd.apply(&mut grid);

    assert!(grid.get(5, 5).unwrap().is_empty());

    cmd.undo(&mut grid);
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
}

#[test]
fn test_clear_grid_command() {
    let mut grid = Grid::new(10, 10);
    grid.set_char(5, 5, 'X');

    let mut cmd = ClearGridCommand::new();
    cmd.apply(&mut grid);

    assert!(grid.get(5, 5).unwrap().is_empty());

    cmd.undo(&mut grid);
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
}

#[test]
fn test_draw_command() {
    let mut grid = Grid::new(10, 10);
    let ops = vec![
        DrawOp::new(0, 0, 'A'),
        DrawOp::new(1, 0, 'B'),
        DrawOp::new(2, 0, 'C'),
    ];

    let mut cmd = DrawCommand::new(ops);
    cmd.apply(&mut grid);

    assert_eq!(grid.get(0, 0).unwrap().ch, 'A');
    assert_eq!(grid.get(1, 0).unwrap().ch, 'B');
    assert_eq!(grid.get(2, 0).unwrap().ch, 'C');

    cmd.undo(&mut grid);

    assert!(grid.get(0, 0).unwrap().is_empty());
    assert!(grid.get(1, 0).unwrap().is_empty());
    assert!(grid.get(2, 0).unwrap().is_empty());
}

#[test]
fn test_draw_command_can_merge() {
    let cmd1 = DrawCommand::new(vec![DrawOp::new(0, 0, 'A')]);
    let cmd2 = DrawCommand::new(vec![DrawOp::new(1, 0, 'B')]);

    assert!(cmd1.can_merge(&cmd2));
}

#[test]
fn test_composite_command() {
    let mut grid = Grid::new(10, 10);

    let mut composite = CompositeCommand::new("Multiple changes");
    composite.add(Box::new(SetCellCommand::new(0, 0, Cell::new('A'))));
    composite.add(Box::new(SetCellCommand::new(1, 0, Cell::new('B'))));
    composite.add(Box::new(SetCellCommand::new(2, 0, Cell::new('C'))));

    composite.apply(&mut grid);

    assert_eq!(grid.get(0, 0).unwrap().ch, 'A');
    assert_eq!(grid.get(1, 0).unwrap().ch, 'B');
    assert_eq!(grid.get(2, 0).unwrap().ch, 'C');

    composite.undo(&mut grid);

    assert!(grid.get(0, 0).unwrap().is_empty());
    assert!(grid.get(1, 0).unwrap().is_empty());
    assert!(grid.get(2, 0).unwrap().is_empty());
}
