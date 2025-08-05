#![allow(non_snake_case)]
#![allow(dead_code)]

mod game_board;
mod game_logic;
mod myshapes;
mod players;
mod tiles;
mod tiles_render;
mod unused;

use std::f32::consts::PI;

use bevy::{input::mouse::MouseButtonInput, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_eventlistener::prelude::*;
use bevy_mod_picking::{debug::DebugPickingMode, prelude::*};

use game_board::*;
use game_logic::*;
use tiles::*;
use tiles_render::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<MouseButtonInput>()
        .add_event::<ScaledDragEvent>()
        .add_event::<PlaceholderTileDropEvent>()
        // Disable bevy_mod_picking logging.
        .insert_resource(State::new(DebugPickingMode::Disabled))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_camera,
                zoom_camera,
                spawn_tile,
                handle_scaled_drag_event.run_if(on_event::<ScaledDragEvent>()),
                handle_tile_drop_event.run_if(on_event::<PlaceholderTileDropEvent>()),
                rotate_tile,
                print_game_data,
                print_tile_data,
                spawn_placeholder_tile,
            ),
        )
        .run();
}

fn move_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<
        (&mut Camera, &mut Transform, &mut OrthographicProjection),
        With<MainCamera>,
    >,
) {
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        camera_q.single_mut().1.translation.x -= 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        camera_q.single_mut().1.translation.x += 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        camera_q.single_mut().1.translation.y += 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        camera_q.single_mut().1.translation.y -= 5. * camera_q.single_mut().2.scale;
    }
    //
    // else if keys.any_pressed([KeyCode::Q]) {
    //     camera_q.single_mut().1.rotate_z(-0.01);
    // } else if keys.any_pressed([KeyCode::E]) {
    //     camera_q.single_mut().1.rotate_z(0.01);
    // }
}

fn zoom_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<&mut OrthographicProjection, (With<Camera>, With<MainCamera>)>,
) {
    if keys.any_pressed([KeyCode::ControlLeft]) {
        if camera_q.single_mut().scale > 0.2 {
            camera_q.single_mut().scale -= 0.01;
        }
    } else if keys.any_pressed([KeyCode::Space]) {
        if camera_q.single_mut().scale < 3. {
            camera_q.single_mut().scale += 0.01;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let game_tile_data: GameTileData = Default::default();
    commands.insert_resource::<GameTileData>(game_tile_data.clone());

    let mut initial_unspawned_tiles: Vec<TileIndex> = vec![];
    for (i, _tile) in game_tile_data.all_tiles.into_iter().enumerate() {
        initial_unspawned_tiles.push(i);
    }

    commands.insert_resource::<GameplayData>(GameplayData {
        spawned_tiles: vec![],
        unspawned_tiles: initial_unspawned_tiles,
        board_tile_matrix: Default::default(),
        board_tile_matrix_inverse: Default::default(),
        next_placeholder_index: PLACEHOLDER_TILE_OFFSET,
        board_area_graph: Default::default(),
        area_index_to_area_graph_index: Default::default(),
    });

    commands.spawn((
        Camera2dBundle::default(),
        RaycastPickCamera::default(),
        MainCamera,
    ));

    // TODO: add with proper meeple data
    let mut y_offset: f32 = 150.0;
    for color in [
        Color::DARK_GRAY,
        Color::CYAN,
        Color::FUCHSIA,
        Color::MAROON,
        Color::INDIGO,
    ] {
        let mut x_offset: f32 = 300.0;
        for _i in 0..8 {
            let mut transform = Transform::from_translation(Vec3::new(x_offset, y_offset, 5.0));
            transform.rotate_z(PI / 4.0);
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::Quad::new(Vec2::new(18., 18.)).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: transform,
                    ..default()
                },
                // MeepleIndex,
                PickableBundle::default(),    // Makes the entity pickable
                RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
                On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                On::<Pointer<DragEnd>>::target_insert(Pickable {
                    should_block_lower: true,
                    should_emit_events: true,
                }), // Re-enable picking
                On::<Pointer<Drag>>::send_event::<ScaledDragEvent>(),
            ));

            x_offset += 40.0;
        }
        y_offset -= 50.0
    }
}

fn print_game_data(keys: Res<Input<KeyCode>>, game_data: Res<GameplayData>) {
    if keys.just_pressed(KeyCode::P) {
        game_data.print();
    }
}

fn print_tile_data(keys: Res<Input<KeyCode>>, tile_data: Res<GameTileData>) {
    if keys.just_pressed(KeyCode::BracketLeft) {
        println!("all_tiles: {:?}", tile_data.all_tiles);
        println!("all_areas: {:?}", tile_data.all_areas);
    }
}
