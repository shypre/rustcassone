#![allow(non_snake_case)]

mod myshapes;
mod tiles;
mod tiles_render;
mod game_board;

use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseButtonInput, input::ButtonState, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::{
    system_param::{Raycast, RaycastSettings},
    *,
};
use petgraph::{
    data::FromElements,
    stable_graph::{NodeIndex, StableGraph},
    Undirected, dot::{Dot, Config},
};
use rand::Rng;
use std::collections::HashMap;

use tiles::*;
use tiles_render::*;
use game_board::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<MouseButtonInput>()
        .add_event::<ScaledDragEvent>()
        .add_event::<TileDragEvent>()
        .add_event::<PlaceholderTileDropEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_button_events,
                move_camera,
                zoom_camera,
                spawn_tile,
                handle_scaled_drag_event.run_if(on_event::<ScaledDragEvent>()),
                handle_tile_drag_event.run_if(on_event::<TileDragEvent>()),
                handle_tile_drop_event.run_if(on_event::<PlaceholderTileDropEvent>()),
                rotate_tile,
                print_game_data,
                print_tile_data,
                spawn_placeholder_tile,
            ),
        )
        .run();
}

fn mouse_button_events(mut mousebtn_evr: EventReader<MouseButtonInput>) {
    for ev in mousebtn_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                println!("Mouse button press: {:?}", ev.button);
            }
            ButtonState::Released => {
                println!("Mouse button release: {:?}", ev.button);
            }
        }
    }
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

fn spawn_tile(
    keys: Res<Input<KeyCode>>,
    window: Query<&Window>,
    mut tile_data: ResMut<GameTileData>,
    mut gameplay_data: ResMut<GameplayData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q: Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    if keys.just_pressed(KeyCode::T) {
        if gameplay_data.unspawned_tiles.len() == 0 {
            println!("No more tiles");
            return;
        }

        let next_tile_index: usize =
            rand::thread_rng().gen_range(0..gameplay_data.unspawned_tiles.len());
        let next_tile: TileIndex = gameplay_data.unspawned_tiles[next_tile_index];

        println!("spawn tile {:?}", next_tile);
        create_tile(
            next_tile,
            window.single(),
            tile_data.into_inner(),
            &mut commands,
            &mut meshes,
            &mut materials,
            camera_q.single().1,
            camera_q.single().3,
        );

        gameplay_data.unspawned_tiles.remove(next_tile_index);
        gameplay_data.spawned_tiles.push(next_tile);

        // First tile needs a placeholder tile to start the graph with.
        if gameplay_data.next_placeholder_index == PLACEHOLDER_TILE_OFFSET {
            try_insert_placeholder_tiles_to_board(
                &mut gameplay_data,
                123456,
                window,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut q,
                camera_q,
            );
        }
    }
}

fn spawn_placeholder_tile(
    keys: Res<Input<KeyCode>>,
    window: Query<&Window>,
    mut tile_data: ResMut<GameTileData>,
    mut gameplay_data: ResMut<GameplayData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q: Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    if keys.just_pressed(KeyCode::Y) {
        println!("spawn placeholder tile");
        try_insert_placeholder_tiles_to_board(
            &mut gameplay_data,
            123456, // nonexistant value
            window,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut q,
            camera_q,
        );
    }
}

fn rotate_tile(
    keys: Res<Input<KeyCode>>,
    window: Query<&Window>,
    mut raycast: Raycast,
    mut tile_data: ResMut<GameTileData>,
    mut q: Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    if keys.just_pressed(KeyCode::R) {
        println!("rotate tile");

        // let mut ray_pos = camera_q.single_mut().2.translation;

        let mouse_world_pos: Vec2 =
            mouse_to_world_position(window.single(), camera_q.single().1, camera_q.single().3);

        let ray_pos = mouse_world_pos.extend(100.0);
        let ray_dir = Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        };
        let ray = Ray3d::new(ray_pos, ray_dir);

        let hits = raycast.cast_ray(ray, &RaycastSettings::default());
        for hit in hits {
            let q_tile_info: TileEntityInfo;
            if let Ok((_e, mut _e_transform, tile_info)) = q.get(hit.0) {
                q_tile_info = tile_info.clone();
                for mut tile in q.iter_mut() {
                    if tile.2.tile_idx == q_tile_info.tile_idx {
                        tile.1.rotate_z(PI / 2.0);
                    }
                }
                return;
            }
        }
        println!("No targets found for rotation");
    }
}

// Nonexistant adjacent_tile results in placing at mouse position
fn try_insert_placeholder_tiles_to_board(
    mut gameplay_data: &mut ResMut<GameplayData>,
    origin_tile_index: TileIndex,
    window: Query<&Window>,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut q: &mut Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    let origin_tile_graph_index = gameplay_data
        .tile_index_to_tile_graph_index
        .get(&origin_tile_index)
        .cloned();

    if !origin_tile_graph_index.is_some() {
        let next_placeholder_index = gameplay_data.next_placeholder_index;

        create_placeholder_tile(
            next_placeholder_index,
            commands,
            meshes,
            materials,
            mouse_to_world_position(window.single(), camera_q.single().1, camera_q.single().3),
        );

        let tile_graph_index = gameplay_data
            .board_tile_graph
            .add_node(next_placeholder_index);
        gameplay_data
            .tile_index_to_tile_graph_index
            .insert(next_placeholder_index, tile_graph_index);

        gameplay_data.next_placeholder_index += 1;
        return;
    }

    let edges = gameplay_data
        .board_tile_graph
        .edges(origin_tile_graph_index.unwrap());

    let mut found_directions: Vec<TileDirection> = vec![];
    let mut empty_directions: Vec<TileDirection> = vec![];
    for e in edges {
        found_directions.push(e.weight().clone());
    }
    for dir in vec![
        TileDirection::UP,
        TileDirection::RIGHT,
        TileDirection::DOWN,
        TileDirection::LEFT,
    ] {
        if !found_directions.contains(&dir) {
            empty_directions.push(dir);
        }
    }

    let mut found: bool = false;
    let mut tile_pos: Vec2 = Default::default();
    for (_e, transform, tile_info) in q.iter() {
        if tile_info.tile_idx == origin_tile_index {
            tile_pos = Vec2 {
                x: transform.translation.x,
                y: transform.translation.y,
            };
            found = true;
            break;
        }
    }
    assert!(found == true);

    for dir in empty_directions {
        let next_placeholder_index = gameplay_data.next_placeholder_index;

        create_placeholder_tile(
            next_placeholder_index,
            commands,
            meshes,
            materials,
            tile_pos + direction_to_offset(dir),
        );

        let placeholder_tile_graph_index = gameplay_data
            .board_tile_graph
            .add_node(next_placeholder_index);
        gameplay_data
            .tile_index_to_tile_graph_index
            .insert(next_placeholder_index, placeholder_tile_graph_index);
        gameplay_data.next_placeholder_index += 1;

        gameplay_data.board_tile_graph.add_edge(
            placeholder_tile_graph_index,
            origin_tile_graph_index.unwrap(),
            dir,
        );

        return;
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

    let mut starting_board_tile_graph: StableGraph<TileIndex, TileDirection, Undirected> =
        Default::default();
    // let starting_node_index = starting_board_tile_graph.add_node(PLACEHOLDER_TILE_OFFSET);
    let mut starting_tile_graph_index_to_tile_index: HashMap<TileIndex, NodeIndex> =
        Default::default();
    // starting_tile_graph_index_to_tile_index.insert(PLACEHOLDER_TILE_OFFSET, starting_node_index);

    commands.insert_resource::<GameplayData>(GameplayData {
        spawned_tiles: vec![],
        unspawned_tiles: initial_unspawned_tiles,
        board_tile_graph: starting_board_tile_graph,
        next_placeholder_index: PLACEHOLDER_TILE_OFFSET,
        tile_index_to_tile_graph_index: starting_tile_graph_index_to_tile_index,
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

fn spawn_square_on_click(
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    use bevy::input::ButtonState;

    for ev in mousebtn_evr.iter() {
        if ev.state == ButtonState::Pressed && ev.button == MouseButton::Right {
            let coords =
                mouse_to_world_position(window.single(), camera_q.single().0, camera_q.single().2);

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::Quad::new(Vec2::new(50., 50.)).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::RED)),
                    transform: Transform::from_translation(Vec3::new(coords.x, coords.y, 0.)),
                    ..default()
                },
                PickableBundle::default(),    // Makes the entity pickable
                RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
                On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
                On::<Pointer<DragEnd>>::target_insert(Pickable {
                    should_block_lower: true,
                    should_emit_events: true,
                }), // Re-enable picking
                On::<Pointer<Drag>>::send_event::<ScaledDragEvent>(),
            ));
        }
    }
}
