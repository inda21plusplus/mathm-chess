use std::collections::HashMap;

use bevy::{
    prelude::{self as b, DespawnRecursiveExt, IntoSystem},
    utils::HashSet,
};
// use bevy_inspector_egui as bi;

use chess_engine as c;

mod entities;

// colors:
//     227, 237, 234
//     201, 187, 168
//     153, 133, 109
//     97, 114, 122
//     75, 116, 127

fn main() {
    b::App::build()
        .insert_resource(b::WindowDescriptor {
            title: "Tjess? Jes!".into(),
            ..Default::default()
        })
        .insert_resource(b::ClearColor(b::Color::rgb(0.1, 0.1, 0.1)))
        // .insert_resource(b::Msaa { samples: 4 })
        .insert_resource(c::Board::default())
        .add_plugins(b::DefaultPlugins)
        // .add_plugin(bi::WorldInspectorPlugin::default())
        .add_event::<MoveMadeEvent>()
        .init_resource::<PieceMaterials>()
        .add_startup_system(spawn_game_tiles_s.system())
        .add_system(lerp_piece_positions_s.system())
        .add_system(update_piece_squares_s.system())
        .add_system(create_pieces_s.system())
        // .add_system(random_moves_s.system())
        .add_system(mouse_hover_highlights_s.system())
        .add_system(set_square_colors_s.system())
        .add_system(pick_up_piece_s.system())
        .run();
}

const PIECE_Z_OFFSET: f32 = 1.0;
const PIECE_LERP_SPEED: f32 = 80.;
const CAMERA_POS_X: f32 = 35.;
const CAMERA_POS_Y: f32 = 35.;
const CAMERA_POS_Z: f32 = 100.;

enum Square {
    Normal,
    Movable,
    Capturable,
}
#[derive(Clone, Copy)]
struct IsOnSquare(b::Entity);
struct MoveMadeEvent(c::Move);
struct PickedUpPiece;
pub struct PieceMaterials(HashMap<c::Piece, b::Handle<b::ColorMaterial>>);

fn random_moves_s(
    mut board: b::ResMut<c::Board>,
    time: b::Res<b::Time>,
    mut t: b::Local<f32>,
    mut move_made_event: b::EventWriter<MoveMadeEvent>,
) {
    *t += time.delta_seconds();
    if *t < 1. {
        return;
    }
    *t -= 1.;

    use rand::seq::SliceRandom;
    let moves: Vec<c::Move> = board.all_legal_moves().collect();
    if let Some(&move_) = moves.choose(&mut rand::thread_rng()) {
        let _ = board.make_move(move_);
        move_made_event.send(MoveMadeEvent(move_));
    }
}

fn set_square_colors_s(
    mut square_q: b::Query<
        (&Square, &c::Color, &mut b::Handle<b::ColorMaterial>),
        b::Changed<Square>,
    >,
    mut materials: b::ResMut<b::Assets<b::ColorMaterial>>,
) {
    for (square, color, mut material_h) in square_q.iter_mut() {
        *material_h = materials.add(
            match (square, color) {
                (Square::Normal, c::Color::White) => b::Color::rgb_u8(153, 133, 109),
                (Square::Normal, c::Color::Black) => b::Color::rgb_u8(201, 187, 168),
                (Square::Movable, c::Color::White) => b::Color::rgb_u8(97, 114, 122),
                (Square::Movable, c::Color::Black) => b::Color::rgb_u8(75, 116, 127),
                (Square::Capturable, c::Color::White) => b::Color::rgb_u8(153, 133, 109),
                (Square::Capturable, c::Color::Black) => b::Color::rgb_u8(153, 133, 109),
            }
            .into(),
        );
    }
}

fn cursor_to_world_coordinates(window: &b::Window) -> b::Vec2 {
    if let Some(pos) = window.cursor_position() {
        (pos - b::Vec2::new(CAMERA_POS_X, CAMERA_POS_Y)) / window.height() * 0.48
    } else {
        b::Vec2::ZERO
    }
}

fn get_square_position_under_cursor(window: &b::Window) -> Option<c::Position> {
    if let Some(mut cursor_position) = window.cursor_position() {
        cursor_position -= b::Vec2::new(window.width() / 2., window.height() / 2.);
        cursor_position /= window.height();
        cursor_position *= 4. / 0.48;
        cursor_position.y *= -1.;
        cursor_position += b::Vec2::new(4., 4.);
        cursor_position = cursor_position.floor();
        let position = c::Position::new_i8(cursor_position.x as i8, cursor_position.y as i8);
        position
    } else {
        None
    }
}

fn pick_up_piece_s(
    mut commands: b::Commands,
    board: b::Res<c::Board>,
    windows: b::Res<b::Windows>,
    square_q: b::Query<&c::Position, b::With<Square>>,
    piece_q: b::Query<(b::Entity, &IsOnSquare), b::With<c::Piece>>,
    cursor_events: b::Res<b::Input<b::MouseButton>>,
) {
    if !cursor_events.just_pressed(b::MouseButton::Left) {
        return;
    }

    let position = windows
        .get_primary()
        .map(|win| get_square_position_under_cursor(win))
        .flatten();

    if let Some(cursor_pos) = position {
        for (piece_entity, square_entity) in piece_q.iter() {
            let square_pos = square_q.get(square_entity.0).unwrap();
            if *square_pos == cursor_pos {
                commands.entity(piece_entity).insert(PickedUpPiece);
                break;
            }
        }
    }
}

fn mouse_hover_highlights_s(
    board: b::Res<c::Board>,
    windows: b::Res<b::Windows>,
    mut square_q: b::Query<(&c::Position, &mut Square)>,
) {
    let destinations: HashSet<c::Position> = windows
        .get_primary()
        .map(|win| get_square_position_under_cursor(win))
        .flatten()
        .map(|pos| board.moves_at_position(pos))
        .map_or(Default::default(), |moves| moves.collect());
    for (square_position, mut square) in square_q.iter_mut() {
        if destinations.contains(square_position) {
            if board[*square_position].is_some() {
                *square = Square::Capturable;
            } else {
                *square = Square::Movable;
            }
        } else {
            *square = Square::Normal;
        }
    }
}

fn lerp_piece_positions_s(
    windows: b::Res<b::Windows>,
    time: b::Res<b::Time>,
    square_q: b::Query<&b::Transform, (b::With<Square>, b::Without<c::Piece>)>,
    mut piece_q: b::Query<
        (&IsOnSquare, &mut b::Transform, Option<&PickedUpPiece>),
        b::With<c::Piece>,
    >,
) {
    for (square, mut piece_transform, is_picked_up) in piece_q.iter_mut() {
        let square_transform = square_q.get(square.0).unwrap();
        let target = if is_picked_up.is_some() {
            if let Some(target) = windows
                .get_primary()
                .map(|win| cursor_to_world_coordinates(win))
            {
                target.extend(piece_transform.translation.z)
            } else {
                continue;
            }
        } else {
            square_transform.translation + b::Vec3::new(0., 0., PIECE_Z_OFFSET)
        };

        let differance = target - piece_transform.translation;
        if differance.length() < 1. {
            piece_transform.translation = target;
            piece_transform.rotation = b::Quat::from_axis_angle(b::Vec3::X, 0.);
        } else {
            let delta = differance.normalize_or_zero() * time.delta_seconds() * PIECE_LERP_SPEED;
            piece_transform.translation += delta;

            piece_transform.rotation =
                b::Quat::from_axis_angle(b::Vec3::new(0., 0., 1.), differance.z / 10.);
        }
    }
}

fn update_piece_squares_s(
    mut commands: b::Commands,
    square_q: b::Query<(b::Entity, &c::Position), b::With<Square>>,
    mut piece_q: b::Query<(b::Entity, &mut IsOnSquare), b::With<c::Piece>>,
    mut move_made_event: b::EventReader<MoveMadeEvent>,
) {
    for MoveMadeEvent(move_) in move_made_event.iter() {
        let mut new_square = b::Entity::new(0);
        for (square_entity, &position) in square_q.iter() {
            if position == move_.to {
                new_square = square_entity;
                break;
            }
        }
        assert_ne!(new_square.id(), 0);
        for (piece_entity, mut square) in piece_q.iter_mut() {
            let (_, square_position) = square_q.get(square.0).unwrap();
            if *square_position == move_.to {
                commands.entity(piece_entity).despawn_recursive();
            }
            if *square_position == move_.from {
                square.0 = new_square;
            }
        }
    }
}

fn create_pieces_s(
    mut commands: b::Commands,
    board: b::Res<c::Board>,
    piece_materials: b::Res<PieceMaterials>,
    square_q: b::Query<(b::Entity, &b::Transform, &c::Position), b::With<Square>>,
) {
    if !board.is_added() {
        return;
    }

    for (entity, &transform, &position) in square_q.iter() {
        if let Some(piece) = board[position] {
            entities::spawn_piece(&mut commands, piece, &*piece_materials, transform, entity);
        }
    }
}

fn spawn_game_tiles_s(mut commands: b::Commands) {
    commands.spawn_bundle(b::PerspectiveCameraBundle {
        transform: b::Transform {
            translation: b::Vec3::new(CAMERA_POS_X, CAMERA_POS_Y, CAMERA_POS_Z),
            ..Default::default()
        },
        ..Default::default()
    });

    for rank in 0..8 {
        for file in 0..8 {
            let position = c::Position::new_unchecked(file, rank);
            entities::spawn_square(&mut commands, position);
        }
    }
}

impl b::FromWorld for PieceMaterials {
    fn from_world(world: &mut b::World) -> Self {
        let mut map: HashMap<c::Piece, b::Handle<b::ColorMaterial>> = Default::default();

        let asset_server = world.get_resource::<b::AssetServer>().unwrap();
        let mut assets = vec![];

        use c::{piece::Kind::*, Color::*};
        for (color, color_char) in [(White, 'w'), (Black, 'b')] {
            for kind in [Pawn, Bishop, Rook, Knight, King, Queen] {
                let piece = c::Piece { color, kind };
                let path = format!(
                    "pieces/{}{}.png",
                    color_char,
                    piece.kind.name().to_lowercase()
                );

                assets.push((piece, asset_server.load(path.as_str())));
            }
        }

        let mut materials = world
            .get_resource_mut::<b::Assets<b::ColorMaterial>>()
            .unwrap();
        for (piece, asset) in assets {
            let material = materials.add(asset.into());
            map.insert(piece, material);
        }

        Self(map)
    }
}
