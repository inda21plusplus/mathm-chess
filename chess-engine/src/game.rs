use crate::{piece, Board, Color, Error, Move, Piece, Position};

/// # Example use:
/// ```rust
/// # use chess_engine::{piece, Board, Game, GameState, Move};
///
/// let mut game = Game::new(Board::default());
/// loop {
///     # fn get_move() -> Move { Move { from: (0, 0).into(), to: (0, 0).into() } }
///     # fn get_promotion() -> piece::Kind { piece::Kind::Queen }
///     match game.make_move(get_move(), || get_promotion()) {
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
#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Ongoing,
    Checkmate { winner: Color },
    Draw,
}

impl Game {
    pub fn new(board: Board) -> Self {
        Self { board }
    }
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn make_move<M, P>(&mut self, move_: M, pawn_promotion: P) -> Result<GameState, Error>
    where
        M: Into<Move>,
        P: FnOnce() -> piece::Kind,
    {
        let move_ = move_.into();
        if let Some(piece) = self.board[move_.from] {
            if piece.color != self.board.next_to_move() {
                return Err(Error::OtherPlayersTurn);
            }
            if !piece.moves(self.board(), move_.from).any(|p| p == move_.to) {
                return Err(Error::IllegalMove);
            }
        } else {
            return Err(Error::NoPieceToMove);
        }
        self.make_move_unchecked(move_, pawn_promotion)
    }
    /// Make the move without checking if the piece at `move_.from` exists or
    /// can move to `move_.to` legally.
    ///
    /// `checks` signifies whether the opponent will be in check after the move.
    fn make_move_unchecked<M, P>(&mut self, move_: M, pawn_promotion: P) -> Result<GameState, Error>
    where
        M: Into<Move>,
        P: FnOnce() -> piece::Kind,
    {
        let move_: Move = move_.into();
        let piece = self.board[move_.from].unwrap();
        let current_color = self.board.next_to_move();
        let mut captured = self.board[move_.to];

        self.board[move_.to] = self.board[move_.from].take();

        // Handle promotion
        if piece.kind == piece::Kind::Pawn && (move_.to.rank() == 7 || move_.to.rank() == 0) {
            let promoted = Piece::new(current_color, pawn_promotion());
            self.board[move_.to] = Some(promoted);
        }

        // Handle castling
        let delta_file = move_.to.file() as i8 - move_.from.file() as i8;
        if piece.kind == piece::Kind::King && delta_file.abs() == 2 {
            let rook_pos =
                Position::new_unchecked(if delta_file > 0 { 7 } else { 0 }, move_.to.rank());
            let rook_dst_file = move_.to.file() as i8 + -delta_file / 2;
            let rook_dst = Position::new_unchecked(rook_dst_file as u8, move_.to.rank());
            self.board[rook_dst] = self.board[rook_pos].take();
        }

        // Handle castling marking
        match (piece.kind, move_.from.file()) {
            (piece::Kind::King, _) => {
                self.board.cannot_castle_kingside(current_color);
                self.board.cannot_castle_queenside(current_color);
            }
            (piece::Kind::Rook, 0) => self.board.cannot_castle_queenside(current_color),
            (piece::Kind::Rook, 7) => self.board.cannot_castle_kingside(current_color),
            _ => {}
        }

        // Handle en passant capture
        if piece.kind == piece::Kind::Pawn && Some(move_.to) == self.board.en_passant_square() {
            let target_rank = move_.to.rank() as i8 + current_color.backwards();
            let target = Position::new_unchecked(move_.to.file(), target_rank as u8);
            captured = self.board[target].take();
        }

        // Handle en passant marking
        let delta_rank = move_.to.rank() as i8 - move_.from.rank() as i8;
        if piece.kind == piece::Kind::Pawn && delta_rank.abs() == 2 {
            let eps_rank = move_.to.rank() as i8 + current_color.backwards();
            self.board
                .set_en_passant_square(Some(Position::new_unchecked(
                    move_.to.file(),
                    eps_rank as u8,
                )));
        } else {
            self.board.set_en_passant_square(None);
        }

        self.board.switch_next_to_move();
        if captured.is_some() || piece.kind == piece::Kind::Pawn {
            self.board.reset_halfmove_counter();
        }

        let mut has_moves = false;
        'outer: for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new_unchecked(file, rank);
                if let Some(piece) = self.board[pos] {
                    if piece.color == self.board.next_to_move()
                        && piece.moves(&self.board, pos).count() > 0
                    {
                        has_moves = true;
                        break 'outer;
                    }
                }
            }
        }
        if !has_moves {
            if piece::util::threatened_at(
                self.board.get_king_position(self.board.next_to_move()),
                &[],
                &[],
                self.board.next_to_move(),
                &self.board,
            ) {
                Ok(GameState::Checkmate {
                    winner: self.board.next_to_move().other(),
                })
            } else {
                Ok(GameState::Draw)
            }
        } else if self.board.halfmove_counter == 50 {
            Ok(GameState::Draw)
        } else {
            Ok(GameState::Ongoing)
        }
    }
}
