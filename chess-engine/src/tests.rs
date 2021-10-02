use std::collections::HashSet;

use crate::*;

#[test]
fn arabic_parsing() {
    for (input, output) in [
        ("a4a5", Move::from(((0, 4).into(), (0, 3).into()))),
        ("h5h1", ((7, 3).into(), (7, 7).into()).into()),
        ("a1h8", ((0, 7).into(), (7, 0).into()).into()),
        ("b5a7", ((1, 3).into(), (0, 1).into()).into()),
        (
            "a2h8q",
            ((0, 6).into(), (7, 0).into(), piece::Kind::Queen).into(),
        ),
        (
            "e7e8n",
            ((4, 1).into(), (4, 0).into(), piece::Kind::Knight).into(),
        ),
    ] {
        assert_eq!(Move::arabic(input), Ok(output), "at: {}", input);
    }
}

#[test]
fn queen_cant_threaten_king_through_own_pieces() {
    let board = Board::from_fen("7K/8/8/4P1Q/1k/8/8/8 b - - 0 0").unwrap();

    assert!(board
        .all_legal_moves()
        .any(|m| m == Move::arabic("b4b5").unwrap()));
}

#[test]
fn pawn_checkmate() {
    let setup_moves = [
        Move::arabic("c2c4").unwrap(), // white
        Move::arabic("h7h6").unwrap(),
        Move::arabic("c4c5").unwrap(), // white
        Move::arabic("h6h5").unwrap(),
        Move::arabic("c5c6").unwrap(), // white
        Move::arabic("h5h4").unwrap(),
        Move::arabic("c6d7").unwrap(), // white
    ];

    let mut board = Board::default();

    for m in setup_moves {
        assert_eq!(Ok(BoardState::Normal), board.make_move(m));
    }

    let expected = [
        Move::arabic("b8d7").unwrap(),
        Move::arabic("c8d7").unwrap(),
        Move::arabic("d8d7").unwrap(),
        Move::arabic("e8d7").unwrap(),
    ];
    let actual = board.all_legal_moves().collect::<Vec<Move>>();
    assert_eq!(
        expected.iter().copied().collect::<HashSet<Move>>(),
        actual.iter().copied().collect::<HashSet<Move>>(),
        "\n{}\n{}",
        expected
            .iter()
            .fold(String::new(), |acc, m| acc + " " + &m.as_arabic()),
        actual
            .iter()
            .fold(String::new(), |acc, m| acc + " " + &m.as_arabic())
    )
}

#[test]
fn arabic_parsing_fails() {
    assert!(matches!(Move::arabic("a4a"), Err(Error::ParsingError)));
    assert!(matches!(Move::arabic("i2a3"), Err(Error::ParsingError)));
    assert!(matches!(Move::arabic("a2u3"), Err(Error::ParsingError)));
    assert!(matches!(Move::arabic("a4a4 "), Err(Error::ParsingError)));
    assert!(matches!(Move::arabic(" a4a4"), Err(Error::ParsingError)));
    assert!(matches!(Move::arabic("a4a4l"), Err(Error::ParsingError)));
}

#[test]
fn piece_parsing() {
    for color in [Color::White, Color::Black] {
        use piece::Kind::*;
        for (c, kind) in [('p', Pawn), ('r', Rook), ('b', Bishop)] {
            assert_eq!(
                Piece::from_name(if color == Color::Black {
                    c
                } else {
                    c.to_ascii_uppercase()
                }),
                Ok(Piece::new(color, kind)),
            );
        }
    }
}

#[test]
fn piece_parsing_fail() {
    for c in "acdefghijlmostuvwxyzACDEFGHIJLMOSTUVWXYZ".chars() {
        assert!(Piece::from_name(c).is_err())
    }
}

fn perft(board: Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut ans = 0;
    for mut move_ in board.all_legal_moves() {
        if board.missing_promotion(move_) {
            for kind in [
                piece::Kind::Bishop,
                piece::Kind::Knight,
                piece::Kind::Queen,
                piece::Kind::Rook,
            ] {
                move_.promotion = Some(kind);
                let mut new_board = board.clone();
                new_board.make_move(move_).unwrap();
                ans += perft(new_board, depth - 1);
            }
        } else {
            let mut new_board = board.clone();
            new_board.make_move(move_).unwrap();
            ans += perft(new_board, depth - 1);
        }
    }
    ans
}

#[test]
fn perft_1() {
    let board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    assert_eq!(20, perft(board.clone(), 1));
    assert_eq!(400, perft(board.clone(), 2));
    assert_eq!(8902, perft(board.clone(), 3));
    // assert_eq!(197281, perft(board.clone(), 4));
}

#[test]
fn perft_2() {
    let board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(48, perft(board.clone(), 1));
    assert_eq!(2039, perft(board.clone(), 2));
    // assert_eq!(97862, perft(board.clone(), 3));
    // assert_eq!(4085603, perft(board.clone(), 4));
}

#[test]
fn perft_3() {
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    assert_eq!(14, perft(board.clone(), 1));
    // assert_eq!(191, perft(board.clone(), 2));
    // assert_eq!(2812, perft(board.clone(), 3));
}

#[test]
fn perft_4() {
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
        .unwrap();
    assert_eq!(6, perft(board.clone(), 1));
    assert_eq!(264, perft(board.clone(), 2));
    // assert_eq!(9467, perft(board.clone(), 3));
}

#[test]
fn perft_5() {
    let board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    assert_eq!(44, perft(board.clone(), 1));
    assert_eq!(1486, perft(board.clone(), 2));
    // assert_eq!(62379, perft(board.clone(), 3));
}

#[test]
fn perft_6() {
    let board =
        Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .unwrap();
    assert_eq!(46, perft(board.clone(), 1));
    assert_eq!(2079, perft(board.clone(), 2));
    // assert_eq!(89890, perft(board.clone(), 3));
}

#[test]
fn few_simple_moves() {
    let mut board = Board::default();
    assert_eq!(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        board.to_fen(),
    );
    assert_eq!(
        Ok(BoardState::Normal),
        board.make_move(Move::arabic("e2e4").unwrap())
    );
    assert_eq!(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        board.to_fen(),
    );
    assert_eq!(
        Ok(BoardState::Normal),
        board.make_move(Move::arabic("c7c5").unwrap())
    );
    assert_eq!(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        board.to_fen(),
    );
    assert_eq!(
        Ok(BoardState::Normal),
        board.make_move(Move::arabic("g1f3").unwrap())
    );
    assert_eq!(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        board.to_fen(),
    );
}

#[test]
fn default_board() {
    let board = Board::default();
    assert_eq!(
        *board.tiles(),
        [
            [
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Rook,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Knight,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Bishop,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Queen,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::King,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Bishop,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Knight,
                }),
                Some(Piece {
                    color: Color::Black,
                    kind: crate::piece::Kind::Rook,
                }),
            ],
            [Some(Piece {
                color: Color::Black,
                kind: crate::piece::Kind::Pawn,
            }); 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [None; 8],
            [Some(Piece {
                color: Color::White,
                kind: crate::piece::Kind::Pawn,
            }); 8],
            [
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Rook,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Knight,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Bishop,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Queen,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::King,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Bishop,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Knight,
                }),
                Some(Piece {
                    color: Color::White,
                    kind: crate::piece::Kind::Rook,
                }),
            ],
        ]
    );
    assert_eq!(board.next_to_move(), Color::White);
    assert_eq!(board.can_castle_kingside(Color::White), true);
    assert_eq!(board.can_castle_queenside(Color::White), true);
    assert_eq!(board.can_castle_kingside(Color::Black), true);
    assert_eq!(board.can_castle_queenside(Color::Black), true);
    assert_eq!(board.en_passant_square(), None);
    assert_eq!(board.halfmove_counter(), 0);
    assert_eq!(board.move_number(), 1);
}
