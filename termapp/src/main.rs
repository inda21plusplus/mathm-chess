use chess_engine::{piece, Board, Game, GameState, Move, Position};
use std::{
    io::{BufRead, Write},
    str::FromStr,
};

fn main() {
    let stdin = std::io::stdin();

    let mut fen = String::new();
    let board = loop {
        print!("Initial board state (empty for none) > ");
        std::io::stdout().lock().flush().unwrap();
        fen.clear();
        stdin.read_line(&mut fen).unwrap();
        break if fen == "" {
            Board::default()
        } else {
            match Board::from_fen(&fen) {
                Ok(board) => board,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            }
        };
    };

    let mut game = Game::new(board);
    print!("{}", game.board().to_string());
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

        let mut m = match Move::arabic(line.trim()) {
            Ok(m) => m,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        if game.missing_promotion(m) {
            m.promotion = loop {
                break match piece::Kind::from_name(lines.next().unwrap().chars().next().unwrap()) {
                    Ok(kind) => Some(kind),
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
            }
        }

        match game.make_move(m) {
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
