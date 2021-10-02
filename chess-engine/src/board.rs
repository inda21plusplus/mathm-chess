use std::{fmt, ops};

use crate::{
    piece::{self, util::threatened_at},
    Color, Error, Move, Piece, Position,
};

mod fen;

/// Represents the state of a chess board.
///
/// Note: the `Board` must always represent a valid state. Some methods might
/// panic if the is not the case.
///
/// # Example
/// ```rust
/// # use chess_engine::{piece, Board, Game, GameState, Move};
///
/// let mut game = Game::new(Board::default());
/// loop {
///     # fn get_move() -> Move { ((0, 0).into(), (0, 0).into()).into() }
///     # fn get_promotion() -> piece::Kind { piece::Kind::Queen }
///     let mut move_ = get_move();
///     if game.missing_promotion(move_) {
///         move_.promotion = Some(get_promotion());
///     }
///     match game.make_move(move_) {
///         Ok(GameState::Ongoing) => {}
///         Ok(s) => {
///             println!("{:?}", s);
///             break;
///         }
///         Err(err) => {
///             println!("{}", err);
///             break;
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    tiles: [[Option<Piece>; 8]; 8],
    next_to_move: Color,
    can_castle_white_kingside: bool,
    can_castle_white_queenside: bool,
    can_castle_black_kingside: bool,
    can_castle_black_queenside: bool,
    en_passant_square: Option<Position>,
    halfmove_counter: u16,
    move_number: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardState {
    Normal,
    Checkmate { winner: Color },
    Draw,
}
impl Board {
    /// Indicates if the is missing the additional `promotion` field when it's
    /// needed. If it's not needed for the move, or if it's already set, false
    /// is returned.
    ///
    /// For invalid moves, there is no guarantee about what this returns. It
    /// will probably be `false` though.
    pub fn missing_promotion<M>(&self, move_: M) -> bool
    where
        M: Into<Move>,
    {
        let move_: Move = move_.into();
        if move_.promotion.is_some() {
            return false;
        }
        let piece = match self[move_.from] {
            Some(piece) => piece,
            None => return false,
        };
        piece.kind == piece::Kind::Pawn && move_.to.rank() == piece.color.other().home_rank()
    }
    pub fn all_legal_moves<'s>(&'s self) -> impl Iterator<Item = Move> + 's {
        (0..8)
            .map(move |rank| {
                (0..8).map(move |file| {
                    let from = Position::new_unchecked(file, rank);
                    (from, self[from])
                })
            })
            .flatten()
            .flat_map(move |(from, piece)| match piece {
                Some(piece) if piece.color == self.next_to_move() => Some((from, piece)),
                _ => None,
            })
            .map(move |(from, piece)| {
                piece.moves(self, from).map(move |to| Move {
                    from,
                    to,
                    promotion: None,
                })
            })
            .flatten()
    }
    pub fn make_move<M>(&mut self, move_: M) -> Result<BoardState, Error>
    where
        M: Into<Move>,
    {
        let move_ = move_.into();
        if let Some(piece) = self[move_.from] {
            if piece.color != self.next_to_move() {
                return Err(Error::OtherPlayersTurn);
            }
            if !piece.moves(self, move_.from).any(|p| p == move_.to) {
                return Err(Error::IllegalMove);
            }
        } else {
            return Err(Error::NoPieceToMove);
        }
        self.make_move_unchecked(move_)
    }
    /// Make the move without checking if the piece at `move_.from` exists or
    /// can move to `move_.to` legally.
    ///
    /// `checks` signifies whether the opponent will be in check after the move.
    fn make_move_unchecked<M>(&mut self, move_: M) -> Result<BoardState, Error>
    where
        M: Into<Move>,
    {
        let move_: Move = move_.into();
        let piece = self[move_.from].unwrap();
        let current_color = self.next_to_move();
        let mut captured = self[move_.to];

        self[move_.to] = self[move_.from].take();

        // Handle promotion
        if piece.kind == piece::Kind::Pawn && (move_.to.rank() == 7 || move_.to.rank() == 0) {
            let promoted_kind = match move_.promotion {
                None | Some(piece::Kind::King) | Some(piece::Kind::Pawn) => {
                    return Err(Error::RequiresPromotion)
                }
                Some(kind) => kind,
            };
            let promoted = Piece::new(current_color, promoted_kind);
            self[move_.to] = Some(promoted);
        }

        // Handle castling
        let delta_file = move_.to.file() as i8 - move_.from.file() as i8;
        if piece.kind == piece::Kind::King && delta_file.abs() == 2 {
            let rook_pos =
                Position::new_unchecked(if delta_file > 0 { 7 } else { 0 }, move_.to.rank());
            let rook_dst_file = move_.to.file() as i8 + -delta_file / 2;
            let rook_dst = Position::new_unchecked(rook_dst_file as u8, move_.to.rank());
            self[rook_dst] = self[rook_pos].take();
        }

        // Handle castling marking
        match (piece.kind, move_.from.file()) {
            (piece::Kind::King, _) => {
                self.cannot_castle_kingside(current_color);
                self.cannot_castle_queenside(current_color);
            }
            (piece::Kind::Rook, 0) => self.cannot_castle_queenside(current_color),
            (piece::Kind::Rook, 7) => self.cannot_castle_kingside(current_color),
            _ => {}
        }
        if move_.to == Position::new_unchecked(0, current_color.other().home_rank()) {
            self.cannot_castle_queenside(current_color.other());
        }
        if move_.to == Position::new_unchecked(7, current_color.other().home_rank()) {
            self.cannot_castle_kingside(current_color.other());
        }

        // Handle en passant capture
        if piece.kind == piece::Kind::Pawn && Some(move_.to) == self.en_passant_square() {
            let target_rank = move_.to.rank() as i8 + current_color.backwards();
            let target = Position::new_unchecked(move_.to.file(), target_rank as u8);
            captured = self[target].take();
        }

        // Handle en passant marking
        let delta_rank = move_.to.rank() as i8 - move_.from.rank() as i8;
        if piece.kind == piece::Kind::Pawn && delta_rank.abs() == 2 {
            let eps_rank = move_.to.rank() as i8 + current_color.backwards();
            self.set_en_passant_square(Some(Position::new_unchecked(
                move_.to.file(),
                eps_rank as u8,
            )));
        } else {
            self.set_en_passant_square(None);
        }

        self.switch_next_to_move();
        if captured.is_some() || piece.kind == piece::Kind::Pawn {
            self.reset_halfmove_counter();
        }

        let mut has_moves = false;
        'outer: for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new_unchecked(file, rank);
                if let Some(piece) = self[pos] {
                    if piece.color == self.next_to_move() && piece.moves(&self, pos).count() > 0 {
                        has_moves = true;
                        break 'outer;
                    }
                }
            }
        }
        if !has_moves {
            if piece::util::threatened_at(
                self.get_king_position(self.next_to_move()),
                &[],
                &[],
                self.next_to_move(),
                &self,
            ) {
                Ok(BoardState::Checkmate {
                    winner: self.next_to_move().other(),
                })
            } else {
                Ok(BoardState::Draw)
            }
        } else if self.halfmove_counter == 50 {
            Ok(BoardState::Draw)
        } else {
            Ok(BoardState::Normal)
        }
    }
    pub fn tiles(&self) -> &[[Option<Piece>; 8]; 8] {
        &self.tiles
    }
    /// Signifies wich color in next up to make a move. Starts as `Color::White`
    /// on a `Default` board
    pub fn next_to_move(&self) -> Color {
        self.next_to_move
    }
    /// Sets `next_to_move` to the other color and increments `move_number` if
    /// `next_to_move` was black before call. Also increments
    /// `halfmove_counter`
    fn switch_next_to_move(&mut self) {
        self.halfmove_counter += 1;
        if self.next_to_move() == Color::Black {
            self.move_number += 1;
        }
        self.next_to_move = self.next_to_move.other();
    }
    /// Retrieves the tile where a pawn can move to capture another pawn that
    /// just moved two ranks
    pub fn en_passant_square(&self) -> Option<Position> {
        self.en_passant_square
    }
    fn set_en_passant_square(&mut self, eps: Option<Position>) {
        self.en_passant_square = eps;
    }
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.can_castle_white_kingside,
            Color::Black => self.can_castle_black_kingside,
        }
    }
    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.can_castle_white_queenside,
            Color::Black => self.can_castle_black_queenside,
        }
    }
    /// Marks that `color` can no longer castle on the kingside. Can be called
    /// even if it was not possible before calling (but will have no effect)
    fn cannot_castle_kingside(&mut self, color: Color) {
        match color {
            Color::White => self.can_castle_white_kingside = false,
            Color::Black => self.can_castle_black_kingside = false,
        }
    }
    /// Marks that `color` can no longer castle on the queenside. Can be called
    /// even if it was not possible before calling (but will have no effect)
    fn cannot_castle_queenside(&mut self, color: Color) {
        match color {
            Color::White => self.can_castle_white_queenside = false,
            Color::Black => self.can_castle_black_queenside = false,
        }
    }
    fn reset_halfmove_counter(&mut self) {
        self.halfmove_counter = 0;
    }
    pub fn halfmove_counter(&self) -> u16 {
        self.halfmove_counter
    }
    pub fn move_number(&self) -> u16 {
        self.move_number
    }
    /// Returns the position of the king with the color `color`.
    pub fn get_king_position(&self, color: Color) -> Position {
        let mut pos = Position::new_unchecked(0, 0);
        while self[pos]
            != Some(Piece {
                color,
                kind: piece::Kind::King,
            })
        {
            if pos.file() == 7 {
                pos = Position::new_unchecked(0, pos.rank() + 1)
            } else {
                pos = Position::new_unchecked(pos.file() + 1, pos.rank())
            }
        }
        pos
    }
    pub fn is_in_check(&self) -> bool {
        threatened_at(
            self.get_king_position(self.next_to_move),
            &[],
            &[],
            self.next_to_move,
            self,
        )
    }
}

impl ops::Index<Position> for Board {
    type Output = Option<Piece>;
    fn index(&self, p: Position) -> &Self::Output {
        &self.tiles[p.rank() as usize][p.file() as usize]
    }
}

impl ops::IndexMut<Position> for Board {
    fn index_mut(&mut self, p: Position) -> &mut Self::Output {
        &mut self.tiles[p.rank() as usize][p.file() as usize]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}  A B C D E F G H",
            self.tiles()
                .iter()
                .enumerate()
                .map(|(i, row)| format!(
                    "{}{}\n",
                    8 - i,
                    row.iter()
                        .map(|p| format!(" {}", p.as_ref().map(Piece::emoji).unwrap_or('.')))
                        .collect::<String>()
                ))
                .collect::<String>()
        )
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}
