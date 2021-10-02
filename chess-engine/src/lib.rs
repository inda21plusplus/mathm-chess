//! For example uses, see `Game`

#![deny(warnings)]

mod board;
mod error;
pub mod piece;
mod util;

pub use board::{Board, BoardState};
pub use error::Error;
pub use piece::Piece;
pub use util::{Color, Move, Position};

#[cfg(test)]
mod tests;
