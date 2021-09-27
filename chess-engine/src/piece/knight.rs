use crate::{Board, Color, Position};

use super::util::threatened_at;
use super::Piece;

pub fn checks(at: Position, color: Color, board: &Board) -> bool {
    let king_pos = board.get_king_position(color.other());
    let delta_file = (king_pos.file() as i8 - at.file() as i8).abs();
    let delta_rank = (king_pos.rank() as i8 - at.rank() as i8).abs();

    delta_file == 1 && delta_rank == 2 || delta_file == 2 && delta_rank == 1
}

pub struct Moves<'b> {
    board: &'b Board,
    from: Position,
    color: Color,
    state: u8,
}

impl<'b> Moves<'b> {
    pub fn new(board: &'b Board, from: Position) -> Self {
        Self {
            board,
            from,
            color: board[from].unwrap().color,
            state: 0,
        }
    }
}

impl<'b> Iterator for Moves<'b> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (x, y) = [
                (2, -1),
                (1, -2),
                (-1, -2),
                (-2, -1),
                (-2, 1),
                (-1, 2),
                (1, 2),
                (2, 1),
            ]
            .get(self.state as usize)?;
            self.state += 1;

            let pos = match Position::new_i8(self.from.file() as i8 + x, self.from.rank() as i8 + y)
            {
                Some(pos) => pos,
                None => {
                    continue;
                }
            };

            let checkcheck = |pos| {
                !threatened_at(
                    self.board.get_king_position(self.color),
                    &[self.from],
                    &[pos],
                    self.color,
                    self.board,
                )
            };
            break match self.board[pos] {
                None if checkcheck(pos) => Some(pos),
                Some(Piece { color: c, .. }) if c != self.color && checkcheck(pos) => Some(pos),
                _ => continue,
            };
        }
    }
}
