use chess_engine::{piece, Board, Game, GameState, Move, Position};
use std::{io::BufRead, str::FromStr};

fn main() {
    let mut game = Game::new(Board::default());
    print!("{}", game.board().to_string());
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines().map(|line| line.unwrap());
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.len() == 2 {
            let pos = match Position::from_str(line) {
                Ok(pos) => pos,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
            match game.board()[pos] {
                Some(piece) => println!(
                    "{}",
                    piece
                        .moves(game.board(), pos)
                        .fold(String::new(), |acc, p| format!("{} {}", acc, p))
                ),
                None => {}
            }
            continue;
        }

        let m = match Move::arabic(line.trim()) {
            Ok(m) => m,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        match game.make_move(m, || loop {
            break match piece::Kind::from_name(lines.next().unwrap().chars().next().unwrap()) {
                Ok(kind) => kind,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
        }) {
            Ok(GameState::Ongoing) => (),
            Ok(GameState::Draw) => {
                println!("Draw!");
                return;
            }
            Ok(GameState::Checkmate { winner }) => {
                println!("Checkmate! {:?} wins", winner);
                return;
            }
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        print!("{}", game.board().to_string());
    }
}
