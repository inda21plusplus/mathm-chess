use std::{fmt, str::FromStr};

use crate::{piece, Board, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<piece::Kind>,
}

impl From<(Position, Position)> for Move {
    fn from((from, to): (Position, Position)) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }
}

impl From<(Position, Position, piece::Kind)> for Move {
    fn from((from, to, promotion): (Position, Position, piece::Kind)) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}

impl Move {
    /// Returns the move represented by `s` in arabic notation, e.g. "a4c6".
    ///
    /// If `s` is not valid arabic notation, `Err(Error::ParsingError)` is returned.
    pub fn arabic(s: &str) -> Result<Self, Error> {
        match s.len() {
            0..=3 => Err(Error::ParsingError),
            4 => Ok(Self {
                from: s[..2].parse()?,
                to: s[2..4].parse()?,
                promotion: None,
            }),
            5.. => Ok(Self {
                from: s[..2].parse()?,
                to: s[2..4].parse()?,
                promotion: Some(s[4..5].parse()?),
            }),
            _ => unreachable!(),
        }
    }
    /// Returns the move as a string in arabic notation, e.g. "h8a1"
    pub fn as_arabic(&self) -> String {
        format!("{}{}", self.from, self.to)
    }
    /// # Note
    /// Doesn't work
    /// # If it worked:
    /// Returns the move as a string in algebraic notation, e.g. "Qh4xe1"
    /// `board` must be the state of the board *before* the move is made.
    ///
    /// For moves that lead to pawn promotions, `promotion` must be set to the
    /// kind of piece the pawn was turned in to. If not, None will be returned.
    /// For all other moves, it is not used and should be set to None.
    ///
    /// If no piece exists on the `move`'s `from` tile, None is returned.
    pub fn as_algebraic(&self, board: &Board, promotion: Option<piece::Kind>) -> Option<String> {
        let piece = board[self.from]?;

        if piece.kind == piece::Kind::King && self.to.file() == self.from.file() + 2 {
            // Kingside castling
            return Some("0-0".to_owned());
        } else if piece.kind == piece::Kind::King && self.to.file() == self.to.file() - 2 {
            // Queenside castling
            return Some("0-0-0".to_owned());
        }

        let kind_part = if piece.kind == piece::Kind::Pawn {
            "".to_owned()
        } else {
            piece.kind.name().to_string()
        };
        let from_file_part = if (0..8u8)
            .map(|rank| Position::new_unchecked(self.from.file(), rank))
            .map(|pos| board[pos] == Some(piece))
            .any(|b| b)
        {
            ((self.from.file() + b'a') as char).to_string()
        } else {
            "".to_owned()
        };
        let from_rank_part = if (0..8u8)
            .map(|file| Position::new_unchecked(file, self.from.rank()))
            .map(|pos| board[pos] == Some(piece))
            .any(|b| b)
        {
            ((b'8' - self.from.rank()) as char).to_string()
        } else {
            "".to_owned()
        };
        let capture_part = if board[self.to].is_some() { "x" } else { "" };
        let dest_part = self.to;
        let promotion_part = promotion.map_or("".to_owned(), |kind| kind.name().to_string());
        let check_part = if piece.checks(self.to, board) {
            "+"
        } else {
            ""
        };

        Some(format!(
            "{kind}{from_file}{from_rank}{capture}{destination}{promotion}{check}",
            kind = kind_part,
            from_file = from_file_part,
            from_rank = from_rank_part,
            capture = capture_part,
            destination = dest_part,
            promotion = promotion_part,
            check = check_part,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(&self) -> Color {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
    pub fn forwards(&self) -> i8 {
        match self {
            Self::White => -1,
            Self::Black => 1,
        }
    }
    pub fn backwards(&self) -> i8 {
        -self.forwards()
    }
    pub fn home_rank(&self) -> u8 {
        match self {
            Self::White => 7,
            Self::Black => 0,
        }
    }
    pub fn home_pawn_rank(&self) -> u8 {
        match self {
            Self::White => 6,
            Self::Black => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Position {
    file: u8,
    rank: u8,
}

impl Position {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }
    pub fn new_i8(file: i8, rank: i8) -> Option<Self> {
        if 0 <= file && file < 8 && 0 <= rank && rank < 8 {
            Some(Self {
                file: file as u8,
                rank: rank as u8,
            })
        } else {
            None
        }
    }
    pub fn new_unchecked(file: u8, rank: u8) -> Self {
        Self { file, rank }
    }
    pub fn new_i8_unchecked(file: i8, rank: i8) -> Self {
        Self {
            file: file as u8,
            rank: rank as u8,
        }
    }
    pub fn file(&self) -> u8 {
        self.file
    }
    pub fn rank(&self) -> u8 {
        self.rank
    }
    pub fn get_file(c: char) -> u8 {
        c as u8 + b'a'
    }
    pub fn get_rank(c: char) -> u8 {
        b'8' - c as u8
    }
}

impl From<(u8, u8)> for Position {
    fn from((file, rank): (u8, u8)) -> Self {
        Position { file, rank }
    }
}

impl FromStr for Position {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        if s.len() > 2 {
            return Err(Error::ParsingError);
        }
        let file = match s.get(0) {
            c @ Some(b'a'..=b'h') => c.unwrap() - b'a',
            _ => return Err(Error::ParsingError),
        };
        let rank = match s.get(1) {
            c @ Some(b'1'..=b'8') => b'8' - c.unwrap(),
            _ => return Err(Error::ParsingError),
        };
        Ok(Self { file, rank })
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (self.file + b'a') as char,
            (b'8' - self.rank) as char,
        )
    }
}
