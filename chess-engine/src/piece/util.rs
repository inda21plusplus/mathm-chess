use crate::{Board, Color, Position};

use super::Kind;

/// Indicates if a piece at `position` with color `color` can be captured.
///
/// Note: Ignores en passant rules.
///
/// If `ignore_piece_at` is `Some`, that square will be treated as empty.
///
/// If `treat_pos_as_wall` is `Some`, a capture that require moving over that
/// square wont count.
///
/// Also, the piece in question does not have to be at `position` in `board`.
pub fn threatened_at(
    position: Position,
    treat_as_empty: &[Position],
    treat_as_occupied: &[Position],
    color: Color,
    board: &Board,
) -> bool {
    for (x, y) in [
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
        (1, 2),
        (2, 1),
    ] {
        let pos = match Position::new_i8(position.file() as i8 + x, position.rank() as i8 + y) {
            Some(pos) => pos,
            None => continue,
        };
        if treat_as_empty.contains(&pos) {
            continue;
        }
        if board[pos].map_or(false, |piece| {
            piece.kind == Kind::Knight && piece.color == color.other()
        }) {
            return true;
        }
    }
    for (x, y, k) in [
        (0, 1, Kind::Rook),
        (1, 0, Kind::Rook),
        (0, -1, Kind::Rook),
        (-1, 0, Kind::Rook),
        (1, 1, Kind::Bishop),
        (1, -1, Kind::Bishop),
        (-1, -1, Kind::Bishop),
        (-1, 1, Kind::Bishop),
    ] {
        for i in 1..8 {
            let pos = match Position::new_i8(
                position.file() as i8 + i * x,
                position.rank() as i8 + i * y,
            ) {
                Some(pos) => pos,
                None => break,
            };
            if treat_as_empty.contains(&pos) {
                continue;
            }
            if treat_as_occupied.contains(&pos) {
                break;
            }
            if let Some(piece) = board[pos] {
                if piece.color == color {
                    break;
                }
                if piece.kind == k || piece.kind == Kind::Queen {
                    return true;
                }
            }
        }
    }
    for (x, y) in [(-1, color.forwards()), (1, color.forwards())] {
        let pos = match Position::new_i8(position.file() as i8 + x, position.rank() as i8 + y) {
            Some(pos) => pos,
            None => continue,
        };
        if treat_as_empty.contains(&pos) {
            continue;
        }
        if board[pos].map_or(false, |piece| {
            piece.kind == Kind::Pawn && piece.color == color.other()
        }) {
            return true;
        }
    }
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
        let pos = match Position::new_i8(position.file() as i8 + x, position.rank() as i8 + y) {
            Some(pos) => pos,
            None => continue,
        };
        if treat_as_empty.contains(&pos) {
            continue;
        }
        if board[pos].map_or(false, |piece| {
            piece.kind == Kind::King && piece.color == color.other()
        }) {
            return true;
        }
    }
    false
}

pub fn floating_checks(deltas: &[(i8, i8)], at: Position, color: Color, board: &Board) -> bool {
    for (x, y) in deltas {
        for i in 1..8 {
            let pos = match Position::new_i8(at.file() as i8 + i * x, at.rank() as i8 + i * y) {
                Some(pos) => pos,
                None => break,
            };

            if let Some(piece) = board[pos] {
                if piece.color == color.other() && piece.kind == Kind::King {
                    return true;
                } else {
                    break;
                }
            }
        }
    }
    false
}

pub struct Moves<'b> {
    board: &'b Board,
    from: Position,
    color: Color,
    dir_index: u8,
    dist: i8,
    dirs: &'static [(i8, i8)],
}

impl<'b> Moves<'b> {
    pub fn new(board: &'b Board, from: Position, dirs: &'static [(i8, i8)]) -> Self {
        Self {
            board,
            from,
            color: board[from].unwrap().color,
            dir_index: 0,
            dist: 1,
            dirs,
        }
    }
}

impl<'b> Iterator for Moves<'b> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.get(self.dir_index as usize)?;
            let pos = match Position::new_i8(
                self.from.file() as i8 + dir.0 * self.dist,
                self.from.rank() as i8 + dir.1 * self.dist,
            ) {
                Some(pos) => pos,
                None => {
                    self.dir_index += 1;
                    self.dist = 1;
                    continue;
                }
            };

            break match self.board[pos].map(|p| p.color) {
                None => {
                    self.dist += 1;
                    if threatened_at(
                        self.board.get_king_position(self.color),
                        &[self.from],
                        &[pos],
                        self.color,
                        self.board,
                    ) {
                        continue;
                    }
                    Some(pos)
                }
                Some(c) => {
                    self.dir_index += 1;
                    self.dist = 1;
                    if c == self.color
                        || threatened_at(
                            self.board.get_king_position(self.color),
                            &[self.from],
                            &[pos],
                            self.color,
                            self.board,
                        )
                    {
                        continue;
                    }
                    Some(pos)
                }
            };
        }
    }
}
