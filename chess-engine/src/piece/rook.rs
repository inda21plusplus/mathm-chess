use crate::{Board, Color, Position};

use super::util::{self, floating_checks, floating_moves};

const DELTAS: &[(i8, i8)] = &[(0, 1), (1, 0), (0, -1), (-1, 0)];

pub struct Moves<'b>(util::Moves<'b>);

impl<'b> Moves<'b> {
    pub fn new(board: &'b Board, from: Position) -> Self {
        Moves(util::Moves::new(board, from, DELTAS))
    }
}

impl<'b> Iterator for Moves<'b> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn checks(at: Position, color: Color, board: &Board) -> bool {
    floating_checks(DELTAS, at, color, board)
}

pub fn append_moves(board: &Board, from: Position, dst: &mut Vec<Position>) {
    floating_moves(DELTAS, board, from, dst)
}
