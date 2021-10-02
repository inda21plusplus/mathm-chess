//! For example uses, see `Game`

#![deny(warnings)]

mod board;
mod error;
mod game;
pub mod piece;
pub mod util;

pub use board::Board;
pub use error::Error;
pub use game::{Game, GameState};
pub use piece::Piece;
pub use util::{Color, Move, Position};

#[cfg(test)]
mod tests;
