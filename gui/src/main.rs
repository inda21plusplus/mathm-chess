use std::{collections::HashMap, f32::consts::PI};

use bevy::prelude::{self as b, IntoSystem};
use bevy_inspector_egui as bi;

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
        .insert_resource(b::Msaa { samples: 4 })
        .insert_resource(c::Board::default())
        .add_plugins(b::DefaultPlugins)
        .add_plugin(bi::WorldInspectorPlugin::default())
        .add_event::<MoveMadeEvent>()
        .init_resource::<PieceMaterials>()
        .add_startup_system(spawn_game_ui_s.system())
        .add_system(lerp_piece_positions_s.system())
        .add_system(update_piece_squares_s.system())
        .add_system(create_pieces_s.system())
        .run();
}

const PIECE_Z_OFFSET: f32 = 0.1;
const PIECE_LERP_SPEED: f32 = 40.;

struct Square;
#[derive(Clone, Copy)]
struct IsOnSquare(b::Entity);
struct MoveMadeEvent(c::Move);
pub struct PieceMaterials(HashMap<c::Piece, b::Handle<b::ColorMaterial>>);

fn lerp_piece_positions_s(
    time: b::Res<b::Time>,
    square_q: b::Query<&b::Transform, (b::With<Square>, b::Without<c::Piece>)>,
    mut piece_q: b::Query<(&IsOnSquare, &mut b::Transform), b::With<c::Piece>>,
) {
    for (square, mut piece_transform) in piece_q.iter_mut() {
        let square_transform = square_q.get(square.0).unwrap();
        let target = square_transform.translation + b::Vec3::new(0., 0., PIECE_Z_OFFSET);

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
    square_q: b::Query<&c::Position, b::With<Square>>,
    mut piece_q: b::Query<&mut IsOnSquare, b::With<c::Piece>>,
    mut move_made_event: b::EventReader<MoveMadeEvent>,
) {
    for MoveMadeEvent(move_) in move_made_event.iter() {
        let mut new_square = IsOnSquare(b::Entity::new(0));
        for square in piece_q.iter_mut() {
            let square_position = square_q.get(square.0).unwrap();
            if *square_position == move_.to {
                new_square = *square;
                break;
            }
        }
        for mut square in piece_q.iter_mut() {
            let square_position = square_q.get(square.0).unwrap();
            if *square_position == move_.from {
                *square = new_square;
                break;
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

fn spawn_game_ui_s(
    mut commands: b::Commands,
    mut materials: b::ResMut<b::Assets<b::ColorMaterial>>,
) {
    commands.spawn_bundle(b::PerspectiveCameraBundle {
        transform: b::Transform {
            translation: b::Vec3::new(35., -40., 45.),
            rotation: b::Quat::from_axis_angle(b::Vec3::X, PI * 0.3),
            ..Default::default()
        },
        ..Default::default()
    });

    let white_material = materials.add(b::Color::rgb_u8(153, 133, 109).into());
    let black_material = materials.add(b::Color::rgb_u8(201, 187, 168).into());
    for rank in 0..8 {
        for file in 0..8 {
            let position = c::Position::new_unchecked(file, rank);
            entities::spawn_square(
                &mut commands,
                position,
                white_material.clone(),
                black_material.clone(),
            );
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
