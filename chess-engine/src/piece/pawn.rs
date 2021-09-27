use crate::{Board, Color, Position};

use super::util::threatened_at;

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
    fn checkcheck(&self, empty: &[Position], occupied: &[Position]) -> bool {
        !threatened_at(
            self.board.get_king_position(self.color),
            empty,
            occupied,
            self.color,
            self.board,
        )
    }
    fn move_forwards(&self) -> Option<Position> {
        let pos = Position::new(
            self.from.file(),
            (self.from.rank() as i8 + self.color.forwards()) as u8,
        )?;

        if self.board[pos].is_none() && self.checkcheck(&[self.from], &[pos]) {
            Some(pos)
        } else {
            None
        }
    }
    fn move_2_steps(&self) -> Option<Position> {
        if !(self.from.rank() == 6 && self.color == Color::White
            || self.from.rank() == 1 && self.color == Color::Black)
        {
            return None;
        }

        let destination = Position::new(
            self.from.file(),
            (self.from.rank() as i8 + self.color.forwards() * 2) as u8,
        )?;

        let in_between_pos = Position::new_i8_unchecked(
            self.from.file() as i8,
            self.from.rank() as i8 + self.color.forwards(),
        );

        if self.board[in_between_pos].is_none()
            && self.board[destination].is_none()
            && self.checkcheck(&[self.from], &[destination])
        {
            Some(destination)
        } else {
            None
        }
    }
    fn capture(&self, delta_file: i8) -> Option<Position> {
        let pos = Position::new_i8(
            self.from.file() as i8 + delta_file,
            self.from.rank() as i8 + self.color.forwards(),
        )?;

        let is_ep = self.board.en_passant_square() == Some(pos);
        let ep_pawn_pos =
            Position::new_i8_unchecked(pos.file() as i8, pos.rank() as i8 - self.color.forwards());

        let cleared_pieces_normal = &[self.from];
        let cleared_pieces_en_passant = &[self.from, ep_pawn_pos];

        if self.board[pos].map(|p| p.color) == Some(self.color.other())
            || is_ep
                && self.checkcheck(
                    if is_ep {
                        cleared_pieces_en_passant
                    } else {
                        cleared_pieces_normal
                    },
                    &[pos],
                )
        {
            Some(pos)
        } else {
            None
        }
    }
}

impl<'b> Iterator for Moves<'b> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let m = match self.state {
                0 => self.move_forwards(),
                1 => self.move_2_steps(),
                2 => self.capture(-1),
                3 => self.capture(1),
                _ => break None,
            };
            self.state += 1;
            if let Some(m) = m {
                break Some(m);
            }
        }
    }
}

pub fn checks(at: Position, color: Color, board: &Board) -> bool {
    [
        Position::new_i8(at.file() as i8 - 1, at.rank() as i8 + color.forwards()),
        Position::new_i8(at.file() as i8 + 1, at.rank() as i8 + color.forwards()),
    ]
    .iter()
    .flatten()
    .any(|pos| *pos == board.get_king_position(color))
}
