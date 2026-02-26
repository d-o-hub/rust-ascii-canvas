//! History tests.

use ascii_canvas::core::cell::Cell;
use ascii_canvas::core::grid::Grid;
use ascii_canvas::core::commands::{Command, SetCellCommand};
use ascii_canvas::core::history::History;

#[test]
fn test_history_push() {
    let mut history = History::new(10);
    history.push(Box::new(SetCellCommand::new(0, 0, Cell::new('A'))));
    
    assert!(history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.undo_count(), 1);
}

#[test]
fn test_history_undo_redo() {
    let mut grid = Grid::new(10, 10);
    let mut history = History::new(10);
    
    // Push and apply a command
    let mut cmd = SetCellCommand::new(5, 5, Cell::new('X'));
    cmd.apply(&mut grid);
    history.push(Box::new(cmd));
    
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
    
    // Undo
    history.undo(&mut grid);
    assert!(grid.get(5, 5).unwrap().is_empty());
    assert!(!history.can_undo());
    assert!(history.can_redo());
    
    // Redo
    history.redo(&mut grid);
    assert_eq!(grid.get(5, 5).unwrap().ch, 'X');
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_max_depth() {
    let mut history = History::new(5);
    
    for i in 0..10 {
        history.push(Box::new(SetCellCommand::new(i, 0, Cell::new('X'))));
    }
    
    assert_eq!(history.undo_count(), 5);
}

#[test]
fn test_history_clear_on_push() {
    let mut grid = Grid::new(10, 10);
    let mut history = History::new(10);
    
    // Push first command
    let mut cmd1 = SetCellCommand::new(0, 0, Cell::new('A'));
    cmd1.apply(&mut grid);
    history.push(Box::new(cmd1));
    
    // Undo to create redo stack
    history.undo(&mut grid);
    assert_eq!(history.redo_count(), 1);
    
    // Push new command - should clear redo
    let mut cmd2 = SetCellCommand::new(1, 0, Cell::new('B'));
    cmd2.apply(&mut grid);
    history.push(Box::new(cmd2));
    
    assert_eq!(history.redo_count(), 0);
}

#[test]
fn test_history_descriptions() {
    let mut history = History::new(10);
    history.push(Box::new(SetCellCommand::new(0, 0, Cell::new('X'))));
    
    assert_eq!(history.undo_description(), Some("Set cell"));
    assert_eq!(history.redo_description(), None);
    
    let mut grid = Grid::new(10, 10);
    history.undo(&mut grid);
    
    assert_eq!(history.redo_description(), Some("Set cell"));
}
