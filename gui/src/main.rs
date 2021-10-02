#![allow(unused)]
use std::{io::BufRead, str::FromStr, time};

use chess_engine::{
    self,
    piece::{
        Piece,
        Kind::{self,  Bishop, King, Knight, Pawn, Queen, Rook},
        Moves,
    },
    Color,
    Position,
    Board,
    GameState,
    Move,
    
};

use bevy::{
    input::mouse::MouseButtonInput,
    prelude::*,
    ecs::schedule::ShouldRun::{
        self,
        Yes,
        No,
    },
};

const WHITE_PAWN_SPRITE: &str = "big_pieces/wP.png";
const WHITE_ROOK_SPRITE: &str = "big_pieces/wR.png";
const WHITE_KNIGHT_SPRITE: &str = "big_pieces/wN.png";
const WHITE_BISHOP_SPRITE: &str = "big_pieces/wB.png";
const WHITE_KING_SPRITE: &str = "big_pieces/wK.png";
const WHITE_QUEEN_SPRITE: &str = "big_pieces/wQ.png";

const BLACK_PAWN_SPRITE: &str = "big_pieces/bP.png";
const BLACK_ROOK_SPRITE: &str = "big_pieces/bR.png";
const BLACK_KNIGHT_SPRITE: &str = "big_pieces/bN.png";
const BLACK_BISHOP_SPRITE: &str = "big_pieces/bB.png";
const BLACK_KING_SPRITE: &str = "big_pieces/bK.png";
const BLACK_QUEEN_SPRITE: &str = "big_pieces/bQ.png";

const BOARD_SPRITE: &str = "board.png";

const SCALE_FACTOR: f32 = 5. / 8.;

const HIGHLIGHT_COLOR: bevy::prelude::Color = bevy::prelude::Color::rgba(0.1, 0.7,0.5, 0.6);

pub struct Materials {
    board: Handle<ColorMaterial>,
    white_pawn: Handle<ColorMaterial>,
    white_rook: Handle<ColorMaterial>,
    white_knight: Handle<ColorMaterial>,
    white_bishop: Handle<ColorMaterial>,
    white_king: Handle<ColorMaterial>,
    white_queen: Handle<ColorMaterial>,
    black_pawn: Handle<ColorMaterial>,
    black_rook: Handle<ColorMaterial>,
    black_knight: Handle<ColorMaterial>,
    black_bishop: Handle<ColorMaterial>,
    black_king: Handle<ColorMaterial>,
    black_queen: Handle<ColorMaterial>,
}


struct MoveHighlight;

struct PromotionOption;
struct PieceSprite;
struct Game {
    game: chess_engine::Game,
    highlighted_piece: Option<PieceHighlight>,
    is_promoting: bool,
    promotion_pos: Option<Position>,
}


#[derive(Debug)]
struct PieceHighlight {
    piece: chess_engine::Piece,
    pos: Position,
}

struct WindowSize {
    width: f32,
    height: f32,
}

fn display_promotion_screen(
    game: Res<Game>,
) -> ShouldRun {
    if game.is_promoting {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_main(game: Res<Game>) -> ShouldRun {
    if game.is_promoting {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}


fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Tjack!".to_string(),
            width: 650.,
            height: 650.,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system.system())
        .add_startup_stage(
            "game_setup_board",
            SystemStage::single(board_spawn.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_main.system())
                .with_system(handle_mouse_clicks_system.system())
                .with_system(pieces_despawn_system.system())
                .with_system(pieces_spawn.system())
                .with_system(promotion_options_despawn_system.system())
            )
        
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(display_promotion_screen.system())
                .with_system(dummy_system.system())
                .with_system(handle_mouse_click_for_promotion.system())
                
        )
        .run();
}
fn promotion_options_despawn_system(
    mut commands: Commands,
    mut highlighted_piece: Query<(Entity, With<PromotionOption>)>,
) {
    for (highlight_entity, _) in highlighted_piece.iter_mut() {
        commands.entity(highlight_entity).despawn();
    }
}
fn handle_mouse_click_for_promotion(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut game: ResMut<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let window = windows.get_primary().expect("No Primary Window");
    if mouse_input.just_pressed(MouseButton::Left) { 
        let cursor_cordinate = get_cursor_cordinates(window);
        if cursor_cordinate.x < window.width()/2. && cursor_cordinate.y < window.height()/2. {
            exchange_promotion_piece(game, Bishop);
            
        }
        else if cursor_cordinate.x < window.width()/2. && cursor_cordinate.y > window.height()/2. {
            exchange_promotion_piece(game, Queen);
        }
        else if cursor_cordinate.x > window.width()/2. && cursor_cordinate.y < window.height()/2. {
            exchange_promotion_piece(game, Knight);
        }
        else if cursor_cordinate.x > window.width()/2. && cursor_cordinate.y > window.height()/2. {
            exchange_promotion_piece(game, Rook);
        }
    }
    
}

fn exchange_promotion_piece(mut game: ResMut<Game>, kind: Kind) {
    let pos = game.promotion_pos;
    let piece = Piece{
        color: get_other_color(game.game.clone().board().next_to_move()),
        kind
    };
    game.game.unsafe_set_piece(pos.unwrap(), piece);
    game.is_promoting = false;
}

    
//promotion_sprite_spawn(commands, match_piece_to_sprite(Queen, game.game.board().next_to_move(), &materials), 40., 40.);

fn promotion_sprite_spawn(commands: &mut Commands, sprite: Handle<ColorMaterial>, x: f32, y: f32) {
    commands
    .spawn_bundle(SpriteBundle {
        material: sprite,
        transform: Transform {
            translation: Vec3::new(x, y, 50.),
            scale: Vec3::new(0.8, 0.8, 1.),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(PromotionOption);
}


fn dummy_system(
    mut commands: Commands,
    game: Res<Game>,    
    materials: Res<Materials>,
){
    promotion_sprite_spawn(&mut commands, match_piece_to_sprite(Queen, get_other_color(game.game.board().next_to_move()), &materials), -150., 150.);
    promotion_sprite_spawn(&mut commands, match_piece_to_sprite(Rook, get_other_color(game.game.board().next_to_move()), &materials), 150., 150.);
    promotion_sprite_spawn(&mut commands, match_piece_to_sprite(Bishop, get_other_color(game.game.board().next_to_move()), &materials), -150., -150.);
    promotion_sprite_spawn(&mut commands, match_piece_to_sprite(Knight, get_other_color(game.game.board().next_to_move()), &materials), 150., -150.);
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
) {
    let mut window = windows.get_primary_mut().unwrap();
    let mut game = chess_engine::Game::new(chess_engine::Board::default());


    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // create the main resources
    commands.insert_resource(Materials {
        board: materials.add(asset_server.load(BOARD_SPRITE).into()),

        white_pawn: materials.add(asset_server.load(WHITE_PAWN_SPRITE).into()),
        white_rook: materials.add(asset_server.load(WHITE_ROOK_SPRITE).into()),
        white_knight: materials.add(asset_server.load(WHITE_KNIGHT_SPRITE).into()),
        white_bishop: materials.add(asset_server.load(WHITE_BISHOP_SPRITE).into()),
        white_king: materials.add(asset_server.load(WHITE_KING_SPRITE).into()),
        white_queen: materials.add(asset_server.load(WHITE_QUEEN_SPRITE).into()),

        black_pawn: materials.add(asset_server.load(BLACK_PAWN_SPRITE).into()),
        black_rook: materials.add(asset_server.load(BLACK_ROOK_SPRITE).into()),
        black_knight: materials.add(asset_server.load(BLACK_KNIGHT_SPRITE).into()),
        black_bishop: materials.add(asset_server.load(BLACK_BISHOP_SPRITE).into()),
        black_king: materials.add(asset_server.load(BLACK_KING_SPRITE).into()),
        black_queen: materials.add(asset_server.load(BLACK_QUEEN_SPRITE).into()),
    });


    commands.insert_resource(Game{
            game, 
            highlighted_piece: None,
            is_promoting: false,
            promotion_pos: None
    });

    commands.insert_resource(WindowSize {
        width: window.width(),
        height: window.height(),
    });
}

fn board_spawn(mut commands: Commands, materials: Res<Materials>, window_size: Res<WindowSize>) {
    // spawn background sprite
   
    commands.spawn_bundle(SpriteBundle {
        material: materials.board.clone(),
        transform: Transform {
            scale: Vec3::new(SCALE_FACTOR, SCALE_FACTOR, 1.),
            ..Default::default()
        },
        ..Default::default()
    });
}


fn pieces_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
    game: Res<Game>,
    window_size: Res<WindowSize>,
) {
    let tiles = game.game.board().tiles(); // TODO: REFACTOR
    let tile_size = window_size.width / 8.;

    let mut i: usize = 0; // TODO: change to enumerate
    let mut j: usize = 0;

    let mut y: f32 = 290.;
    for row in tiles {
        let mut x: f32 = -280.;
        for tile in row {
            // tile is an Option<Piece>
            let piece = match tile {
                Some(p) => p,
                None => {
                    x += tile_size;
                    j += 1;
                    continue;
                }
            };
            let sprite = match_piece_to_sprite(piece.kind, piece.color, &materials);
            piece_spawn(&mut commands, sprite, x, y);
            x += tile_size;
            j += 1;
        }
        y -= tile_size + 1.;
        i += 1;
    }
}

fn piece_spawn(commands: &mut Commands, sprite: Handle<ColorMaterial>, x: f32, y: f32) {
    commands
        .spawn_bundle(SpriteBundle {
            material: sprite,
            transform: Transform {
                scale: Vec3::new(0.2, 0.2, 1.),
                translation: Vec3::new(x, y, 10.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PieceSprite);
        
}

fn pieces_despawn_system(
    mut commands: Commands,
    mut highlighted_piece: Query<(Entity, With<PieceSprite>)>,
) {
    for (highlight_entity, _) in highlighted_piece.iter_mut() {
        commands.entity(highlight_entity).despawn();
    }
}



fn match_piece_to_sprite(
    kind: Kind,
    color: Color,
    materials: &Res<Materials>,
) -> Handle<ColorMaterial> {
    match (kind, color) {
        (Pawn, Color::White) => materials.white_pawn.clone(),
        (Rook, Color::White) => materials.white_rook.clone(),
        (Knight, Color::White) => materials.white_knight.clone(),
        (Bishop, Color::White) => materials.white_bishop.clone(),
        (King, Color::White) => materials.white_king.clone(),
        (Queen, Color::White) => materials.white_queen.clone(),
        (Pawn, Color::Black) => materials.black_pawn.clone(),
        (Rook, Color::Black) => materials.black_rook.clone(),
        (Knight, Color::Black) => materials.black_knight.clone(),
        (Bishop, Color::Black) => materials.black_bishop.clone(),
        (King, Color::Black) => materials.black_king.clone(),
        (Queen, Color::Black) => materials.black_queen.clone(),
    }
}

fn handle_mouse_clicks_system(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut game: ResMut<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut highlighted_moves: Query<(Entity, With<MoveHighlight>)>,
    mut highlighted_piece: Query<(Entity, With<PieceHighlight>)>,
) {
    
    let window = windows.get_primary().expect("No Primary Window");
    if mouse_input.just_pressed(MouseButton::Left) {
        handle_hightlights(&mut game, &mut materials, &mut commands, &mut highlighted_moves, &mut highlighted_piece, window);
        // Moves only click. No drag and drop
        handle_moves(window, game, commands, highlighted_moves, highlighted_piece);
    }
}

fn handle_moves(
    window: &Window,
    mut game: ResMut<Game>,
    mut commands: Commands,
    mut highlighted_moves: Query<(Entity, With<MoveHighlight>)>,
    mut highlighted_piece: Query<(Entity, With<PieceHighlight>)>
){
    let highlighted_piece = match &game.highlighted_piece {
        Some(piece) => piece,
        None => return,
    };
    let piece = highlighted_piece.piece;
    let pos = highlighted_piece.pos;

    let cursor_cordinate = get_cursor_cordinates(window);
    let cursor_pos = get_position_from_cursor(cursor_cordinate, window);

    let b = game.game.board().clone();
    let moves = piece.moves(&b, pos);
    
    for dest_pos in moves {
        if cursor_pos == dest_pos {
            println!("Good Move");
            // Test
            

            let m = Move::from((pos, dest_pos));

            match game.game.make_move(m, || Queen) {
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

            let destination_piece = game.game.board()[dest_pos].unwrap();
            println!("new piece {:?}", destination_piece);
            if piece.kind != destination_piece.kind {
                // Promotion happend 
                game.is_promoting = true;
                game.promotion_pos = Some(dest_pos);
            }
            game.highlighted_piece = None;
            print!("{}", game.game.board().to_string());

        }
    }
}

fn handle_hightlights(
    mut game: &mut ResMut<Game>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut commands: &mut Commands,
    mut highlighted_moves: &mut Query<(Entity, With<MoveHighlight>)>,
    mut highlighted_piece: &mut Query<(Entity, With<PieceHighlight>)>,
    window: &Window,
) {
    highlight_moves_despawn_system(&mut commands, highlighted_moves);
    highlight_piece_despawn_system(&mut commands, highlighted_piece);

    
    let cursor_pos = get_cursor_cordinates(window);
    let pos = get_position_from_cursor(cursor_pos, window);
    

    let piece = match game.game.board()[pos] {
        Some(p) => p,
        None => return,
    }; 

    
    if piece.color == game.game.board().next_to_move() {
        game.highlighted_piece = Some(PieceHighlight {
            piece,
            pos,
        });
        let (piece_x, piece_y) = get_window_pos_from_position(pos, window);
        highlight_piece_spawn_system(&mut commands, &mut materials, piece_x, piece_y, piece, pos);
        
        // Highlight Moves
        let moves = piece.moves(game.game.board(), pos);
        for dest_pos in moves {
            let (x, y ) = get_window_pos_from_position(dest_pos, window);
            highlight_moves_spawn_system(&mut commands, &mut materials, x, y);
            
        }
    }
}

fn get_cursor_cordinates(window: &Window) -> Vec2 {
    let cursor_pos =  match window.cursor_position() {
        Some(pos) => pos,
        None => panic!("No cursor pos at click"),
    };
    cursor_pos
}

fn highlight_piece_spawn_system(
    mut commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32,
    y: f32,
    piece: Piece,
    pos: Position,
) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(HIGHLIGHT_COLOR.into()),
        sprite: Sprite::new(Vec2::new(75., 75.)),
        transform: Transform {
            translation: Vec3::new(x, y, 5.),
            ..Default::default()
        },
    ..Default::default()
    })
    .insert(PieceHighlight {
        piece,
        pos,
    });
}

fn highlight_piece_despawn_system(
    mut commands: &mut Commands,
    mut highlighted_piece: &mut Query<(Entity, With<PieceHighlight>)>,
) {
    for (highlight_entity, _) in highlighted_piece.iter_mut() {
        commands.entity(highlight_entity).despawn();
    }
}

fn highlight_moves_spawn_system(
    mut commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32,
    y: f32,
) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(HIGHLIGHT_COLOR.into()),
        sprite: Sprite::new(Vec2::new(30., 30.)),
        transform: Transform {
            translation: Vec3::new(x, y, 20.),
            ..Default::default()
        },
    ..Default::default()
    })
    .insert(MoveHighlight);
}

fn highlight_moves_despawn_system(
    mut commands: &mut Commands,
    mut highlighted_moves: &mut Query<(Entity, With<MoveHighlight>)>,
) {
    for (highlight_entity, _) in highlighted_moves.iter_mut() {
        commands.entity(highlight_entity).despawn();
    }
}

fn get_position_from_cursor(cursor_pos: Vec2, window: &Window) -> Position {
    // Window is between (30, 30) and (670, 670)
    // origin is bottom left
    let file = (cursor_pos.x / (window.width()/8.)) as u8;
    let rank = 7 - (cursor_pos.y / (window.height()/8.)) as u8;
    let board_pos = match Position::new(file, rank) {
        Some(pos) => pos,
        None => panic!("Invalid Position, file and/or rank not > 0 & <7"),
    };
    board_pos
}

fn get_window_pos_from_position(position: Position, window: &Window) -> (f32, f32) {
    let rank = position.rank();
    let file = position.file();

    let tile_size = (window.width()/8.);

    let x_pos = file as f32 * tile_size - 4. * tile_size + tile_size/2.;

    let y_pos = (7 - rank) as f32 * tile_size - 4. * tile_size+ tile_size/2.;

    (x_pos, y_pos)
}

fn get_promotion_piece() {

}

fn get_other_color(color: Color) -> Color {
    match color {
        Color::White => return Color::Black,
        Color::Black => return Color::White,
        _ => panic!("Color error"),
    }
}