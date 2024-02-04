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
    mut q: Query<(Entity, &mut Transform, Option<&TileEntityInfo>)>,
    // camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
) {
    for event in drop_event.iter() {
        info!(
            "drop_event target: {:?}, dropped: {:?}, hit_position: {:?}",
            event.target, event.dropped, event.position
        );
        let t: Entity;
        let t_transform: Transform;
        let Ok((target, target_transform, _)) = q.get_mut(event.target) else {
            panic!("uh oh not found: {:?}", event.target)
        };
        t = target;
        t_transform = *target_transform;
        // Dropped entity must be a tile.
        let Ok((mut _dropped, mut dropped_transform, is_real_tile)) = q.get_mut(event.dropped)
        else {
            panic!("uh oh not found: {:?}", event.dropped)
        };
        if is_real_tile.is_some() {
            dropped_transform.translation.x = t_transform.translation.x;
            dropped_transform.translation.y = t_transform.translation.y;
            commands.entity(t).despawn();
        } else {
            println!("dropped not a tile onto placeholder, ignoring");
            return;
        }
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
