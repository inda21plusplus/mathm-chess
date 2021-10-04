use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::camera::Camera,
    ui::FocusPolicy,
};
use chess_engine::{piece, Board, BoardState, Color as ChessColor, Move, Piece, Position};
use std::collections::{HashMap, HashSet};

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Chess? Yes!".into(),
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 2 })
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Resources
        .init_resource::<Board>()
        .init_resource::<PieceAssetMap>()
        .insert_resource(UIState::Default)
        // Event types
        .add_event::<BoardUpdateEvent>()
        // Startup systems
        .add_startup_system(setup_game_ui.system())
        // Systems
        .add_system(assign_square_sprites.system())
        .add_system(possible_moves_hover.system())
        .add_system(show_diagnostics.system())
        .add_system(square_state_color.system())
        .add_system(pick_up_piece.system())
        .add_system(put_down_piece.system())
        .add_system(move_picked_up_piece_to_cursor.system())
        .add_system(cancel_move.system())
        .add_system(get_pawn_promotion.system())
        .add_system(update_end_game_text.system())
        //
        .run();
}

struct PieceAssetMap(HashMap<Piece, Handle<ColorMaterial>>);
struct PieceSprite;
#[derive(Clone, Copy)]
struct PawnPromotionOption(piece::Kind);
#[derive(PartialEq, Eq, Clone, Copy)]
enum BoardUpdateEvent {
    MoveMade(Move),
    State(BoardState),
    Other,
}
struct GameEndElement;
struct GameEndText;
struct DiagnosticsInfoText;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UIState {
    Default,
    PickedUpPiece(Entity),
    PromotionAsked(Position, Position),
}
struct PickedUpPieceParent(Entity);
struct PawnPromotionElement(Entity);

#[derive(Clone, Copy)]
enum ChessSquare {
    Normal,
    Movable,
    Capturable,
    Promotable,
}

fn get_pawn_promotion(
    mut state: ResMut<UIState>,
    mut board: ResMut<Board>,
    pawn_promotion_element: Res<PawnPromotionElement>,
    mut pawn_promotion_element_query: Query<(&mut Style, &Children)>,
    pawn_promotion_options_query: Query<(&Interaction, &PawnPromotionOption)>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    let (from, to) = if let UIState::PromotionAsked(from, to) = *state {
        pawn_promotion_element_query
            .get_mut(pawn_promotion_element.0)
            .unwrap()
            .0
            .display = Display::Flex;
        (from, to)
    } else {
        pawn_promotion_element_query
            .get_mut(pawn_promotion_element.0)
            .unwrap()
            .0
            .display = Display::None;
        return;
    };

    let mut chosen = piece::Kind::Pawn;
    for (&interaction, &piece) in pawn_promotion_options_query.iter() {
        if interaction == Interaction::Clicked {
            chosen = piece.0;
        }
    }

    if chosen == piece::Kind::Pawn {
        return;
    }

    assert_eq!(board.next_to_move(), board[from].unwrap().color);

    let move_ = Move {
        from,
        to,
        promotion: Some(chosen),
    };

    *state = UIState::Default;
    board_update_event.send(BoardUpdateEvent::MoveMade(move_));
    board_update_event.send(BoardUpdateEvent::State(board.make_move(move_).unwrap()));
}

fn move_picked_up_piece_to_cursor(
    picked_up_piece_parent: Res<PickedUpPieceParent>,
    mut picked_up_piece_parent_query: Query<&mut Style>,
    windows: Res<Windows>,
    cam_query: Query<&Transform, With<Camera>>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        let window_height = window.height();
        let side_length = window_height * 0.8 / 8.0;

        let cam_tranform = cam_query.single().unwrap();
        let pos = cam_tranform.compute_matrix() * pos.extend(0.0).extend(1.0);
        let pos = Vec2::new(pos.x, pos.y);

        let mut style = picked_up_piece_parent_query
            .get_mut(picked_up_piece_parent.0)
            .unwrap();

        style.position.left = Val::Px(pos.x - side_length / 2.0);
        style.position.bottom = Val::Px(pos.y - side_length / 2.0);
        style.size = Size {
            width: Val::Px(side_length),
            height: Val::Px(side_length),
        }
    }
}

fn pick_up_piece(
    mut commands: Commands,
    query: Query<(Entity, &Interaction, &Position), (Changed<Interaction>, With<PieceSprite>)>,
    mut fp_query: Query<&mut FocusPolicy, With<PieceSprite>>,
    board: Res<Board>,
    mut state: ResMut<UIState>,
    picked_up_piece_parent: Res<PickedUpPieceParent>,
) {
    if *state != UIState::Default {
        return;
    }

    for (entity, &interaction, &pos) in query.iter() {
        if interaction != Interaction::Clicked {
            continue;
        }
        if Some(board.next_to_move()) != board[pos].map(|p| p.color) {
            continue;
        }
        for mut focus_p in fp_query.iter_mut() {
            *focus_p = FocusPolicy::Pass;
        }
        commands
            .entity(entity)
            .remove::<Parent>()
            .insert(Parent(picked_up_piece_parent.0));
        *state = UIState::PickedUpPiece(entity)
    }
}

fn put_down_piece(
    query: Query<(&Interaction, &Position), With<ChessSquare>>,
    mut state: ResMut<UIState>,
    picked_up_piece_query: Query<&Position, Without<ChessSquare>>,
    mut board: ResMut<Board>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    let piece = match *state {
        UIState::PickedUpPiece(p) => p,
        _ => return,
    };
    let from = match picked_up_piece_query.get(piece) {
        Ok(sq) => *sq,
        // I dont know why this ever errors but this seems to work
        Err(_) => {
            *state = UIState::Default;
            board_update_event.send(BoardUpdateEvent::Other);
            return;
        }
    };
    let mut target = None;
    for (&interaction, &sq_spec) in query.iter() {
        if interaction == Interaction::Clicked {
            target = Some(sq_spec);
        }
    }
    let to = match target {
        Some(t) => t,
        None => return,
    };

    let move_ = Move {
        from,
        to,
        promotion: None,
    };

    if to == from {
        *state = UIState::Default;
    } else if board.missing_promotion(move_) {
        *state = UIState::PromotionAsked(from, to);
        board_update_event.send(BoardUpdateEvent::MoveMade(move_));
    } else if let Ok(board_state) = board.make_move(Move {
        from,
        to,
        promotion: None,
    }) {
        *state = UIState::Default;
        board_update_event.send(BoardUpdateEvent::State(board_state));
    }
}

fn cancel_move(
    mouse_input: Res<Input<MouseButton>>,
    kb_input: Res<Input<KeyCode>>,
    mut state: ResMut<UIState>,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
) {
    if !(mouse_input.just_pressed(MouseButton::Right) || kb_input.just_pressed(KeyCode::Escape)) {
        return;
    }
    *state = UIState::Default;
    board_update_event.send(BoardUpdateEvent::Other);
}

fn possible_moves_hover(
    piece_query: Query<(&Interaction, &Position), Changed<Interaction>>,
    mut square_query: Query<(&Position, &mut ChessSquare)>,
    board: Res<Board>,
    state: Res<UIState>,
) {
    if *state != UIState::Default {
        return;
    }

    let mut from = None;
    let mut changed = false;

    for (&interaction, &sq_spec) in piece_query.iter() {
        changed = true;
        if interaction == Interaction::Hovered || interaction == Interaction::Clicked {
            from = Some(sq_spec);
            break;
        }
    }

    if !changed {
        return;
    }

    for (_, mut chess_square) in square_query.iter_mut() {
        *chess_square = ChessSquare::Normal;
    }
    let from = match from {
        Some(hovered) => hovered,
        None => return,
    };
    let destinations: HashSet<Position> = board.moves_at_position(from).collect();

    for (&to, mut chess_square) in square_query.iter_mut() {
        if destinations.contains(&to) {
            if board.missing_promotion(Move {
                from,
                to,
                promotion: None,
            }) {
                *chess_square = ChessSquare::Promotable;
            } else if board[to].is_some() {
                *chess_square = ChessSquare::Capturable;
            } else {
                *chess_square = ChessSquare::Movable;
            }
        }
    }
}

// TODO: cache materials
fn square_state_color(
    mut query: Query<(&Position, &ChessSquare, &mut Handle<ColorMaterial>), Changed<ChessSquare>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (&position, &chess_square, mut material) in query.iter_mut() {
        let is_white = (position.file() + position.rank()) % 2 == 1;
        let color = match (is_white, chess_square) {
            (true, ChessSquare::Normal) => Color::rgb_u8(50, 50, 50),
            (false, ChessSquare::Normal) => Color::rgb_u8(40, 40, 40),
            (true, ChessSquare::Capturable) => Color::rgb_u8(0xd0, 0x87, 0x70),
            (false, ChessSquare::Capturable) => Color::rgb_u8(0xbf, 0x61, 0x6a),
            (true, ChessSquare::Movable) => Color::rgb_u8(0xdb, 0xbb, 0x7b),
            (false, ChessSquare::Movable) => Color::rgb_u8(0xca, 0xa1, 0x75),
            (true, ChessSquare::Promotable) => Color::rgb_u8(0x81, 0xa1, 0xc1),
            (false, ChessSquare::Promotable) => Color::rgb_u8(0x5e, 0x81, 0xac),
        };
        *material = materials.add(color.into());
    }
}

fn show_diagnostics(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<DiagnosticsInfoText>>,
) {
    if let Some(fps) = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .unwrap()
        .average()
    {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("Fps: {:.0}", fps);
    }
}

impl FromWorld for PieceAssetMap {
    fn from_world(world: &mut World) -> Self {
        let mut this = HashMap::default();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut assets = vec![];
        for (color, color_ch) in [(ChessColor::White, 'w'), (ChessColor::Black, 'b')] {
            for kind in [
                piece::Kind::Bishop,
                piece::Kind::King,
                piece::Kind::Knight,
                piece::Kind::Pawn,
                piece::Kind::Queen,
                piece::Kind::Rook,
            ] {
                let path = format!("pieces/{}{}.png", color_ch, kind.name().to_lowercase());
                assets.push((Piece { color, kind }, asset_server.load(path.as_str())));
            }
        }
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        for (piece, asset) in assets {
            let material = materials.add(asset.into());
            this.insert(piece, material);
        }
        Self(this)
    }
}

fn update_end_game_text(
    mut board_update_event: EventReader<BoardUpdateEvent>,
    mut text_query: Query<&mut Text, With<GameEndText>>,
    mut parent_query: Query<&mut Style, With<GameEndElement>>,
) {
    for event in board_update_event.iter() {
        match event {
            BoardUpdateEvent::State(BoardState::Normal) => {
                parent_query.single_mut().unwrap().display = Display::None;
                text_query.single_mut().unwrap().sections[0].value.clear();
            }
            BoardUpdateEvent::State(state) => {
                parent_query.single_mut().unwrap().display = Display::Flex;
                text_query.single_mut().unwrap().sections[0].value = format!("{:?}!", state);
            }
            _ => {}
        }
    }
}

fn assign_square_sprites(
    mut commands: Commands,
    square_query: Query<(Entity, &Position), With<ChessSquare>>,
    sprites: Query<(Entity, &PieceSprite)>,
    board: Res<Board>,
    asset_map: Res<PieceAssetMap>,
    mut board_update_event: EventReader<BoardUpdateEvent>,
) {
    for &event in board_update_event.iter() {
        if event != BoardUpdateEvent::Other {
            continue;
        }
        for (entity, _) in sprites.iter() {
            commands.entity(entity).despawn();
        }

        for (entity, &position) in square_query.iter() {
            if let Some(piece) = board[position] {
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                position_type: PositionType::Absolute,
                                ..Default::default()
                            },
                            material: asset_map.0.get(&piece).unwrap().clone(),
                            ..Default::default()
                        })
                        .insert(Interaction::default())
                        .insert(FocusPolicy::Block)
                        .insert(position.clone())
                        .insert(PieceSprite);
                });
            }
        }
    }
}

fn setup_game_ui(
    mut commands: Commands,
    mut board_update_event: EventWriter<BoardUpdateEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    piece_asset_map: Res<PieceAssetMap>,
) {
    let mut picked_up_piece_parent = Entity::new(0);
    let mut pawn_promotion_element = Entity::new(0);
    let font = asset_server.load("fonts/FiraSans-Bold.otf");
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::rgb_u8(20, 20, 20).into()),
            ..Default::default()
        })
        .with_children(|root| {
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Undefined, Val::Percent(80.0)),
                    aspect_ratio: Some(0.8), // i dont know why but this makes it a square
                    ..Default::default()
                },
                material: materials.add(Color::rgb_u8(40, 40, 40).into()),
                ..Default::default()
            })
            .with_children(|board| {
                board
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            position_type: PositionType::Relative,
                            position: Rect {
                                left: Val::Percent(100.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        material: materials.add(Color::NONE.into()),
                        ..Default::default()
                    })
                    .with_children(|side_proxy| {
                        side_proxy
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    position_type: PositionType::Relative,
                                    position: Rect {
                                        left: Val::Px(40.0),
                                        ..Default::default()
                                    },
                                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                                    ..Default::default()
                                },
                                material: materials.add(Color::rgb_u8(30, 30, 30).into()),
                                ..Default::default()
                            })
                            .with_children(|side_panel| {
                                side_panel
                                    .spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            "",
                                            TextStyle {
                                                font: font.clone(),
                                                font_size: 12.0,
                                                color: Color::WHITE,
                                            },
                                            Default::default(),
                                        ),
                                        ..Default::default()
                                    })
                                    .insert(DiagnosticsInfoText);
                            });
                    });
                // grid
                for rank in 0..8 {
                    for file in 0..8 {
                        board
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: Rect {
                                        bottom: Val::Percent(rank as f32 * 100.0 / 8.0),
                                        left: Val::Percent(file as f32 * 100.0 / 8.0),
                                        ..Default::default()
                                    },
                                    size: Size::new(
                                        Val::Percent(100.0 / 8.0),
                                        Val::Percent(100.0 / 8.0),
                                    ),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Interaction::default())
                            .insert(Position::new(file, rank))
                            .insert(ChessSquare::Normal);
                    }
                }
            });
            root.spawn_bundle(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::RowReverse,
                    border: Rect {
                        top: Val::Px(20.0),
                        left: Val::Px(20.0),
                        right: Val::Px(20.0),
                        bottom: Val::Px(20.0),
                    },
                    // display: Display::None,
                    ..Default::default()
                },
                material: materials.add(Color::rgb_u8(0xb4, 0x8e, 0xad).into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "",
                            TextStyle {
                                font_size: 64.0,
                                color: Color::WHITE,
                                font: font.clone(),
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                vertical: VerticalAlign::Center,
                            },
                        ),
                        ..Default::default()
                    })
                    .insert(GameEndText);
            })
            .insert(GameEndElement);
            picked_up_piece_parent = root
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    material: materials.add(Color::NONE.into()),
                    ..Default::default()
                })
                .insert(FocusPolicy::Pass)
                .id();
            pawn_promotion_element = root
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb_u8(0x88, 0xc0, 0xd0).into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for kind in [
                        piece::Kind::Bishop,
                        piece::Kind::Knight,
                        piece::Kind::Queen,
                        piece::Kind::Rook,
                    ] {
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size {
                                        width: Val::Px(80.0),
                                        height: Val::Px(80.0),
                                    },
                                    ..Default::default()
                                },
                                material: piece_asset_map.0[&Piece {
                                    kind,
                                    color: ChessColor::White,
                                }]
                                    .clone(),
                                ..Default::default()
                            })
                            .insert(Interaction::default())
                            .insert(PawnPromotionOption(kind));
                    }
                })
                .id();
        });

    commands.insert_resource(PickedUpPieceParent(picked_up_piece_parent));
    commands.insert_resource(PawnPromotionElement(pawn_promotion_element));

    board_update_event.send(BoardUpdateEvent::Other);
}
