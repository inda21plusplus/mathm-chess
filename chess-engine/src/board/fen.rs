use std::str::FromStr;

use crate::{error::FenError, piece, Color, Error, Piece, Position};

use super::Board;

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, Error> {
        let mut fen = fen.split_ascii_whitespace();

        let mut found_white_king = false;
        let mut found_black_king = false;

        let mut tiles = [[None; 8]; 8];

        let mut file = 0;
        let mut rank = 0;

        let tiles_part = fen.next().ok_or(Error::FenError(FenError::Pieces))?;
        for c in tiles_part.chars() {
            match c {
                '/' => {
                    rank += 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += c as usize - '0' as usize;
                }
                _ => {
                    let piece = Piece::from_name(c)?;
                    if piece.kind == piece::Kind::King {
                        *match piece.color {
                            Color::White => &mut found_white_king,
                            Color::Black => &mut found_black_king,
                        } = true;
                    }
                    tiles[rank][file] = Some(piece);
                    file += 1;
                }
            }
        }

        let mut board = Board {
            tiles,
            next_to_move: Color::White,
            can_castle_white_kingside: false,
            can_castle_white_queenside: false,
            can_castle_black_kingside: false,
            can_castle_black_queenside: false,
            en_passant_square: None,
            halfmove_counter: 0,
            move_number: 0,
        };

        let next_to_move_part = fen.next().ok_or(Error::FenError(FenError::NextToMove))?;
        board.next_to_move = match next_to_move_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(Error::FenError(FenError::NextToMove)),
        };

        let castling_part = fen.next().ok_or(Error::FenError(FenError::Castling))?;
        for c in castling_part.chars() {
            match c {
                'K' => board.can_castle_white_kingside = true,
                'Q' => board.can_castle_white_queenside = true,
                'k' => board.can_castle_black_kingside = true,
                'q' => board.can_castle_black_queenside = true,
                '-' => {}
                _ => return Err(Error::FenError(FenError::Castling)),
            }
        }

        let en_passant_square_part = fen.next().ok_or(Error::FenError(FenError::EnPassant))?;
        board.en_passant_square = match en_passant_square_part {
            "-" => None,
            ep => match Position::from_str(ep) {
                Ok(p) => Some(p),
                Err(err) => return Err(err),
            },
        };

        let halfmove_counter_part = fen
            .next()
            .ok_or(Error::FenError(FenError::HalfmoveCounter))?;
        board.halfmove_counter = halfmove_counter_part
            .parse()
            .map_err(|_| Error::FenError(FenError::HalfmoveCounter))?;

        let move_number_part = fen.next().ok_or(Error::FenError(FenError::MoveNumber))?;
        board.move_number = move_number_part
            .parse()
            .map_err(|_| Error::FenError(FenError::MoveNumber))?;

        // TODO: Return error if game state is invalid
        if !found_white_king || !found_black_king {
            return Err(Error::InvalidGameState);
        }

        Ok(board)
    }
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in 0..8 {
            let mut skip = 0;
            for file in 0..8 {
                if let Some(piece) = self.tiles[rank][file] {
                    if skip != 0 {
                        fen.push_str(&format!("{}", skip));
                    }
                    fen.push(piece.name());
                    skip = 0;
                } else {
                    skip += 1;
                }
            }
            if rank != 7 {
                if skip != 0 {
                    fen.push_str(&format!("{}", skip));
                }
                fen.push('/');
            }
        }

        fen.push(' ');

        match self.next_to_move() {
            Color::White => fen.push('w'),
            Color::Black => fen.push('b'),
        }

        fen.push(' ');

        if !self.can_castle_white_kingside
            && !self.can_castle_white_queenside
            && !self.can_castle_black_kingside
            && !self.can_castle_black_queenside
        {
            fen.push('-');
        }
        if self.can_castle_white_kingside {
            fen.push('K');
        }
        if self.can_castle_white_queenside {
            fen.push('Q');
        }
        if self.can_castle_black_kingside {
            fen.push('k');
        }
        if self.can_castle_black_queenside {
            fen.push('q');
        }

        fen.push(' ');

        if let Some(ep) = self.en_passant_square {
            fen.push_str(&ep.to_string());
        } else {
            fen.push_str("-");
        }

        fen.push(' ');

        fen.push_str(&format!("{}", self.halfmove_counter));

        fen.push(' ');

        fen.push_str(&format!("{}", self.move_number));

        fen
    }
}
