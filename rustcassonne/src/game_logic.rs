use std::f32::consts::PI;
use std::vec;

use bevy::{
    input::mouse::MouseButtonInput, math::vec4, prelude::*, render::camera::RenderTarget,
    render::mesh::Mesh, sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::backend::HitData;
use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::system_param::Raycast;
use bevy_mod_raycast::system_param::RaycastSettings;
use bevy_mod_raycast::Ray3d;
use rand::Rng;

use crate::game_board::*;
use crate::myshapes::*;
use crate::tiles::*;
use crate::tiles_render::*;

pub fn handle_tile_drop_event(
    mut drop_event: EventReader<PlaceholderTileDropEvent>,
    mut gameplay_data: ResMut<GameplayData>,
    mut q: Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
    // camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in drop_event.iter() {
        println!(
            "drop_event target: {:?}, dropped: {:?}, hit_position: {:?}",
            event.target, event.dropped, event.position
        );
        let t_transform: Transform;
        let t_tile_index: TileIndex;
        {
            let Ok((_target, target_transform, target_tile_info)) = q.get_mut(event.target) else {
                panic!("uh oh not found: {:?}", event.target)
            };
            // Target must be a placeholder tile
            if target_tile_info.tile_idx < PLACEHOLDER_TILE_OFFSET {
                println!("dropped onto a non-placeholder tile, ignoring");
                return;
            }
            t_transform = *target_transform;
            t_tile_index = target_tile_info.tile_idx;
        }
        // Dropped entity must be a non-placeholder tile.
        let Ok((mut _dropped, mut dropped_transform, dropped_tile_info)) = q.get_mut(event.dropped)
        else {
            println!("dropped is not a tile, ignoring");
            return;
        };
        if dropped_tile_info.tile_idx >= PLACEHOLDER_TILE_OFFSET {
            println!("target is a placeholder tile, ignoring");
            return;
        }
        // If dropped is already part of board, ignore.
        if gameplay_data
            .board_tile_matrix_inverse
            .contains_key(&dropped_tile_info.tile_idx)
        {
            println!("dropped is already in game board, ignoring");
            return;
        }

        dropped_transform.translation.x = t_transform.translation.x;
        dropped_transform.translation.y = t_transform.translation.y;

        replace_placeholder_tile_on_board(
            &mut gameplay_data,
            dropped_tile_info.tile_idx,
            t_tile_index,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut q,
        );
    }
}

pub fn spawn_tile(
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
            add_placeholder_tile_and_maybe_init_board(
                &mut gameplay_data,
                window,
                &mut commands,
                &mut meshes,
                &mut materials,
                camera_q,
            );
        }
    }
}

pub fn spawn_placeholder_tile(
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
        add_placeholder_tile_and_maybe_init_board(
            &mut gameplay_data,
            window,
            &mut commands,
            &mut meshes,
            &mut materials,
            camera_q,
        );
    }
}

pub fn rotate_tile(
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
            if let Ok((_e, mut _e_transform, mut tile_info)) = q.get_mut(hit.0) {
                tile_info.dir = rotate_direction(tile_info.dir.clone());
                let tile_idx = tile_info.tile_idx;
                for mut tile in q.iter_mut() {
                    if tile.2.tile_idx == tile_idx {
                        tile.1.rotate_z(PI / 2.0);
                    }
                }
                return;
            }
        }
        println!("No targets found for rotation");
    }
}

fn get_data_of_tile(
    tile_index: TileIndex,
    mut q: &mut Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
) -> Option<(Entity, Transform, TileEntityInfo)> {
    for data in q.iter() {
        if data.2.tile_idx == tile_index {
            let ret = (data.0, data.1.clone(), data.2.clone());
            return Some(ret);
        }
    }
    return None;
}

// Nonexistant adjacent_tile results in placing at mouse position
fn add_placeholder_tile_and_maybe_init_board(
    mut gameplay_data: &mut ResMut<GameplayData>,
    window: Query<&Window>,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    let next_placeholder_index = gameplay_data.next_placeholder_index;
    create_placeholder_tile(
        next_placeholder_index,
        commands,
        meshes,
        materials,
        mouse_to_world_position(window.single(), camera_q.single().1, camera_q.single().3),
    );
    gameplay_data.next_placeholder_index += 1;

    // If origin tile exists already, don't add it to the matrix.
    if gameplay_data
        .board_tile_matrix
        .contains_key(&TileMatrixCoords { x: 0, y: 0 })
    {
        println!("adding additional placeholder tile, it will be free floating");
        return;
    }

    gameplay_data
        .board_tile_matrix
        .insert(TileMatrixCoords { x: 0, y: 0 }, next_placeholder_index);
    gameplay_data
        .board_tile_matrix_inverse
        .insert(next_placeholder_index, TileMatrixCoords { x: 0, y: 0 });
}

fn replace_placeholder_tile_on_board(
    mut gameplay_data: &mut ResMut<GameplayData>,
    replacement_tile_index: TileIndex,
    origin_tile_index: TileIndex,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut q: &mut Query<(Entity, &mut Transform, &mut TileEntityInfo), Without<MainCamera>>,
) {
    let op_origin_tile_data = get_data_of_tile(origin_tile_index, q);
    {
        if op_origin_tile_data.is_none() {
            panic!(
                "origin_tile_index {:?} not found in world:\n",
                origin_tile_index
            );
        }

        if !gameplay_data
            .board_tile_matrix_inverse
            .contains_key(&origin_tile_index)
        {
            panic!(
                "origin_tile_index {:?} not found in matrix:\n{:?}",
                origin_tile_index, gameplay_data.board_tile_matrix_inverse
            );
        }
    }
    let origin_tile_data = op_origin_tile_data.unwrap();

    let op_replacement_tile_data = get_data_of_tile(replacement_tile_index, q);
    {
        if op_replacement_tile_data.is_none() {
            panic!(
                "replacement_tile_index {:?} not found in world:\n",
                replacement_tile_index
            );
        }

        if gameplay_data
            .board_tile_matrix_inverse
            .contains_key(&replacement_tile_index)
        {
            panic!(
                "replacement_tile_index {:?} already in matrix:\n{:?}",
                replacement_tile_index, gameplay_data.board_tile_matrix_inverse
            );
        }
    }
    let target_tile_data = op_replacement_tile_data.unwrap();

    let coords: TileMatrixCoords = gameplay_data
        .board_tile_matrix_inverse
        .get(&origin_tile_data.2.tile_idx)
        .unwrap()
        .clone();
    gameplay_data
        .board_tile_matrix_inverse
        .remove(&origin_tile_data.2.tile_idx);
    gameplay_data
        .board_tile_matrix_inverse
        .insert(target_tile_data.2.tile_idx, coords);
    *gameplay_data.board_tile_matrix.get_mut(&coords).unwrap() = target_tile_data.2.tile_idx;

    // Copy it out before we destroy the placeholder tile.
    let origin_tile_translation = origin_tile_data.1.translation;
    println!("origin_tile_translation: {origin_tile_translation}");
    commands.entity(origin_tile_data.0).despawn();

    // Remove drag function for the new tile.
    // TODO: still crashes when the tile is dragged over placeholder
    commands
        .entity(target_tile_data.0)
        .remove::<On<Pointer<Drag>>>();
    commands
        .entity(target_tile_data.0)
        .remove::<On<Pointer<Drop>>>();
    commands
        .entity(target_tile_data.0)
        .remove::<On<Pointer<DragStart>>>();
    commands
        .entity(target_tile_data.0)
        .remove::<On<Pointer<DragEnd>>>();

    // insert placeholder tiles around new tile
    for dir in NEIGHBOR_COORDS {
        let new_coords = TileMatrixCoords {
            x: coords.x + dir.x,
            y: coords.y + dir.y,
        };
        if !gameplay_data.board_tile_matrix.contains_key(&new_coords) {
            let new_pos = Vec2 {
                x: origin_tile_translation.x + (dir.x as f32 * 180.0),
                y: origin_tile_translation.y + (dir.y as f32 * 180.0),
            };

            let next_placeholder_index = gameplay_data.next_placeholder_index;
            create_placeholder_tile(next_placeholder_index, commands, meshes, materials, new_pos);
            gameplay_data.next_placeholder_index += 1;
            gameplay_data
                .board_tile_matrix
                .insert(new_coords, next_placeholder_index);
            gameplay_data
                .board_tile_matrix_inverse
                .insert(next_placeholder_index, new_coords);
        }
    }
}
