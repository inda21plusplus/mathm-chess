use crate::{Board, Color, Position};

use super::util::{self, floating_checks};

const DELTAS: &[(i8, i8)] = &[
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

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
