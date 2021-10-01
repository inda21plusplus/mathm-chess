use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OtherPlayersTurn,
    NoPieceToMove,
    IllegalMove,
    UnknwonPiece(char),
    ParsingError,
    FenError(FenError),
    InvalidGameState,
    RequiresPromotion,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FenError {
    Pieces,
    NextToMove,
    Castling,
    EnPassant,
    HalfmoveCounter,
    MoveNumber,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OtherPlayersTurn => write!(f, "Other players turn"),
            Self::NoPieceToMove => write!(f, "No piece to move"),
            Self::IllegalMove => write!(f, "Illegal move"),
            Self::UnknwonPiece(c) => write!(f, "Unknown piece {}", c),
            Self::ParsingError => write!(f, "Parsing error"),
            Self::FenError(err) => write!(f, "Fen parsing error at {} part", err),
            Self::InvalidGameState => write!(f, "Invalid game state"),
            Self::RequiresPromotion => write!(f, "Move requires specifying promoted piece kind"),
        }
    }
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pieces => write!(f, "pieces"),
            Self::NextToMove => write!(f, "next to move"),
            Self::Castling => write!(f, "castling"),
            Self::EnPassant => write!(f, "en passant"),
            Self::HalfmoveCounter => write!(f, "halfmove counter"),
            Self::MoveNumber => write!(f, "move number"),
        }
    }
}
