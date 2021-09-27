use crate::{Board, Color, Error, Move, Position};

mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;
pub mod util;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub kind: Kind,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub enum Moves<'b> {
    Pawn(pawn::Moves<'b>),
    Rook(rook::Moves<'b>),
    Knight(knight::Moves<'b>),
    Bishop(bishop::Moves<'b>),
    Queen(queen::Moves<'b>),
    King(king::Moves<'b>),
}

impl<'b> Iterator for Moves<'b> {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Pawn(moves) => moves.next(),
            Self::Rook(moves) => moves.next(),
            Self::Knight(moves) => moves.next(),
            Self::Bishop(moves) => moves.next(),
            Self::Queen(moves) => moves.next(),
            Self::King(moves) => moves.next(),
        }
    }
}

impl Piece {
    pub fn new(color: Color, kind: Kind) -> Self {
        Self { color, kind }
    }
    pub fn emoji(&self) -> char {
        use Color::*;
        use Kind::*;
        match (self.color, self.kind) {
            (White, Pawn) => '♟',
            (Black, Pawn) => '♙',
            (White, Rook) => '♜',
            (Black, Rook) => '♖',
            (White, Knight) => '♞',
            (Black, Knight) => '♘',
            (White, Bishop) => '♝',
            (Black, Bishop) => '♗',
            (White, Queen) => '♛',
            (Black, Queen) => '♕',
            (White, King) => '♚',
            (Black, King) => '♔',
        }
    }
    pub fn from_name(name: char) -> Result<Self, Error> {
        Ok(Piece {
            color: if name.is_ascii_uppercase() {
                Color::White
            } else {
                Color::Black
            },
            kind: Kind::from_name(name)?,
        })
    }
    pub fn name(&self) -> char {
        if self.color == Color::Black {
            self.kind.name().to_ascii_lowercase()
        } else {
            self.kind.name()
        }
    }
    pub fn moves<'b>(&self, board: &'b Board, from: Position) -> Moves<'b> {
        match self.kind {
            Kind::Pawn => Moves::Pawn(pawn::Moves::new(board, from)),
            Kind::Rook => Moves::Rook(rook::Moves::new(board, from)),
            Kind::Knight => Moves::Knight(knight::Moves::new(board, from)),
            Kind::Bishop => Moves::Bishop(bishop::Moves::new(board, from)),
            Kind::Queen => Moves::Queen(queen::Moves::new(board, from)),
            Kind::King => Moves::King(king::Moves::new(board, from)),
        }
    }
    /// Returns whether the piece at `move_.from` legally can move to
    /// `move_.to`.
    #[deprecated]
    pub fn can_move(&self, move_: Move, board: &Board) -> bool {
        // if in check:
        //     if only one piece is checking king:
        //         either:
        //             must capture checker
        //         or:
        //             must place oneself in the way
        //     else:
        //         must move king
        //
        // else:
        //     avoid revealed checks

        #[allow(deprecated)]
        self.get_moves(board, move_.from).contains(&move_.to)
    }
    #[deprecated]
    pub fn get_moves(&self, board: &Board, from: Position) -> Vec<Position> {
        let mut ret = vec![];
        #[allow(deprecated)]
        self.append_moves(board, from, &mut ret);
        ret
    }
    #[deprecated]
    pub fn append_moves(&self, board: &Board, from: Position, dst: &mut Vec<Position>) {
        match self.kind {
            Kind::Pawn => dst.extend(self.moves(board, from)),
            Kind::Rook => rook::append_moves(board, from, dst),
            Kind::Knight => knight::append_moves(board, from, dst),
            Kind::Bishop => bishop::append_moves(board, from, dst),
            Kind::Queen => queen::append_moves(board, from, dst),
            Kind::King => king::append_moves(board, from, dst),
        }
    }
    pub fn checks(&self, at: Position, board: &Board) -> bool {
        match self.kind {
            Kind::Pawn => pawn::checks(at, self.color, board),
            Kind::Rook => rook::checks(at, self.color, board),
            Kind::Knight => knight::checks(at, self.color, board),
            Kind::Bishop => bishop::checks(at, self.color, board),
            Kind::Queen => queen::checks(at, self.color, board),
            Kind::King => king::checks(at, self.color, board),
        }
    }
}

impl Kind {
    pub fn from_name(name: char) -> Result<Self, Error> {
        match name {
            'p' | 'P' => Ok(Self::Pawn),
            'r' | 'R' => Ok(Self::Rook),
            'n' | 'N' => Ok(Self::Knight),
            'b' | 'B' => Ok(Self::Bishop),
            'q' | 'Q' => Ok(Self::Queen),
            'k' | 'K' => Ok(Self::King),
            _ => Err(Error::ParsingError),
        }
    }
    pub fn name(&self) -> char {
        match *self {
            Self::Pawn => 'P',
            Self::Rook => 'R',
            Self::Bishop => 'B',
            Self::Knight => 'N',
            Self::Queen => 'Q',
            Self::King => 'K',
        }
    }
}
