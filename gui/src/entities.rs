use bevy::prelude as b;
use chess_engine as c;

use crate::{IsOnSquare, PieceMaterials, Square, PIECE_Z_OFFSET};

pub fn spawn_piece(
    commands: &mut b::Commands,
    piece: c::Piece,
    piece_materials: &PieceMaterials,
    transform: b::Transform,
    square_entity: b::Entity,
) {
    let mut piece_transform = transform;
    piece_transform.translation.z += PIECE_Z_OFFSET;
    piece_transform.translation.x += (rand::random::<f32>() - 0.5) * 10.;
    piece_transform.translation.y += (rand::random::<f32>() - 0.5) * 10.;
    let z = rand::random::<f32>() * 50. + 150.;
    piece_transform.translation.z += z;
    commands
        .spawn_bundle(b::SpriteBundle {
            sprite: b::Sprite::new(b::Vec2::new(10., 10.)),
            transform: piece_transform,
            visible: b::Visible {
                is_transparent: true,
                is_visible: true,
            },
            material: piece_materials.0.get(&piece).unwrap().clone(),
            ..Default::default()
        })
        .insert(IsOnSquare(square_entity))
        .insert(piece);
}

pub fn spawn_square(commands: &mut b::Commands, position: c::Position) {
    let file = position.file();
    let rank = position.rank();
    let color = if (rank + file) % 2 == 1 {
        c::Color::White
    } else {
        c::Color::Black
    };
    commands
        .spawn_bundle(b::SpriteBundle {
            sprite: b::Sprite::new(b::Vec2::new(10., 10.)),
            transform: b::Transform::from_xyz(10. * file as f32, 70. - 10. * rank as f32, 0.),
            ..Default::default()
        })
        .insert(position)
        .insert(color)
        .insert(Square::Normal);
}
