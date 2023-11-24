#![allow(non_snake_case)]

mod myshapes;
mod tiles;
mod tiles_render;

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
use rand::Rng;
use tiles_render::*;

use tiles::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<MouseButtonInput>()
        .add_event::<GenericDragEvent>()
        .add_event::<TileDragEvent>()
        .add_systems(Startup, (setup))
        .add_systems(
            Update,
            (
                mouse_button_events,
                move_camera,
                zoom_camera,
                spawn_tile,
                handle_generic_drag_event.run_if(on_event::<GenericDragEvent>()),
                handle_tile_drag_event.run_if(on_event::<TileDragEvent>()),
                rotate_tile,
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
            commands,
            meshes,
            materials,
            camera_q.single().1,
            camera_q.single().3,
        );

        gameplay_data.unspawned_tiles.remove(next_tile_index);
        gameplay_data.spawned_tiles.push(next_tile);
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
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform, &GlobalTransform), With<MainCamera>>,
) {
    if keys.just_pressed(KeyCode::Y) {
        println!("spawn placeholder tile");
        create_placeholder_tile(
            window.single(),
            commands,
            meshes,
            materials,
            camera_q.single().1,
            camera_q.single().3,
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

#[derive(Resource, Default, Clone, Debug)]
pub struct GameplayData {
    pub spawned_tiles: Vec<TileIndex>,
    pub unspawned_tiles: Vec<TileIndex>,
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
                On::<Pointer<Drag>>::send_event::<GenericDragEvent>(),
            ));

            x_offset += 40.0;
        }
        y_offset -= 50.0
    }
}

fn print_tile_data(
    keys: Res<Input<KeyCode>>,
    tile_data: Res<GameTileData>,
    game_data: Res<GameplayData>,
) {
    if keys.just_pressed(KeyCode::P) {
        println!("all_tiles: {:?}", tile_data.all_tiles);
        println!("all_areas: {:?}", tile_data.all_areas);
        println!("spawned_tiles: {:?}", game_data.spawned_tiles);
        println!("unspawned_tiles: {:?}", game_data.unspawned_tiles);
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
                On::<Pointer<Drag>>::send_event::<GenericDragEvent>(),
            ));
        }
    }
}
