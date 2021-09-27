use crate::{Board, Color, Position};

use super::util::threatened_at;
use super::Piece;

pub fn checks(_at: Position, _color: Color, _board: &Board) -> bool {
    false
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
                (1, 0),
                (1, -1),
                (0, -1),
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
                (-2, 0),
                (2, 0),
            ]
            .get(self.state as usize)?;
            self.state += 1;

            let checkcheck = |pos| !threatened_at(pos, &[self.from], &[], self.color, self.board);

            let pos = match Position::new_i8(self.from.file() as i8 + x, self.from.rank() as i8 + y)
            {
                Some(pos) => pos,
                None => {
                    continue;
                }
            };

            if *x == -2 {
                let in_between = Position::new_unchecked(self.from.file() - 1, self.from.rank());
                if !self.board.can_castle_queenside(self.color)
                    || !checkcheck(self.from)
                    || self.board[in_between].is_some()
                    || !checkcheck(in_between)
                {
                    continue;
                }
            }

            if *x == 2 {
                let in_between = Position::new_unchecked(self.from.file() + 1, self.from.rank());
                if !self.board.can_castle_kingside(self.color)
                    || !checkcheck(self.from)
                    || self.board[in_between].is_some()
                    || !checkcheck(in_between)
                {
                    continue;
                }
            }

            break match self.board[pos] {
                None if checkcheck(pos) => Some(pos),
                Some(Piece { color: c, .. }) if c != self.color && checkcheck(pos) => Some(pos),
                _ => continue,
            };
        }
    }
}

pub fn append_moves(board: &Board, from: Position, dst: &mut Vec<Position>) {
    let color = board[from].unwrap().color;
    for (x, y) in [
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        let pos = match Position::new_i8(from.file() as i8 + x, from.rank() as i8 + y) {
            Some(pos) => pos,
            None => continue,
        };

        if board[pos].map(|p| p.color) == Some(color) {
            continue;
        }

        if threatened_at(pos, &[from], &[], color, board) {
            continue;
        }

        dst.push(pos);
    }

    if board.can_castle_queenside(color) {
        assert_eq!(from.file(), 4);
    }

    if board.can_castle_queenside(color)
        && board[Position::new_unchecked(from.file() - 3, from.rank())].is_none()
        && !threatened_at(from, &[], &[], color, board)
        && [1, 2]
            .iter()
            .map(|x| Position::new_unchecked(from.file() - x, from.rank()))
            .all(|pos| board[pos].is_none() && !threatened_at(pos, &[], &[], color, board))
    {
        dst.push(Position::new_unchecked(from.file() - 2, from.rank()))
    }

    if board.can_castle_kingside(color)
        && !threatened_at(from, &[], &[], color, board)
        && [1, 2]
            .iter()
            .map(|x| Position::new_unchecked(from.file() + x, from.rank()))
            .all(|pos| board[pos].is_none() && !threatened_at(pos, &[], &[], color, board))
    {
        dst.push(Position::new_unchecked(from.file() + 2, from.rank()))
    }
}
