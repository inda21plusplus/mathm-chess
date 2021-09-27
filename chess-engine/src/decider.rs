use crate::{piece, Board, Error, Game, GameState, Move};

/// # Example use:
/// ```no_run
/// use chess_engine::{piece, Board, Decider, Move};
///
/// struct Application {
///     /* all data needed to run the application, such as windows, etc. */
/// }
///
/// impl Decider for Application {
///     fn get_move(&mut self, board: &Board) -> Move {
///         /* ... */
///         # panic!()
///     }
///     fn get_pawn_promotion(&mut self) -> piece::Kind {
///         /* ... */
///         # panic!()
///     }
/// }
///
/// fn main() {
///     let mut app = Application { /* */ };
///     app.run(Board::default()).unwrap();
/// }
/// ```

pub trait Decider {
    fn get_move(&mut self, board: &Board) -> Move;
    fn get_pawn_promotion(&mut self) -> piece::Kind;

    /// Runs a game of chess with on `board`. Do not overwrite
    fn run(&mut self, board: Board) -> Result<GameState, Error> {
        let mut game = Game::new(board);
        loop {
            let game_state =
                game.make_move(self.get_move(game.board()), || self.get_pawn_promotion())?;
            if game_state != GameState::Ongoing {
                break Ok(game_state);
            }
        }
    }
}
