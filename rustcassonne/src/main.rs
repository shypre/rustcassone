mod tiles;
mod tiles_render;

use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseButtonInput, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::{
    system_param::{Raycast, RaycastSettings},
    *,
};
use tiles_render::*;

use tiles::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<MouseButtonInput>()
        .add_event::<CubeDragEvent>()
        .add_event::<TileDragEvent>()
        .add_systems(Startup, (setup))
        .add_systems(
            Update,
            (
                mouse_button_events,
                move_camera,
                zoom_camera,
                spawn_tile,
                handle_drag_event.run_if(on_event::<CubeDragEvent>()),
                handle_tile_area_drag_event.run_if(on_event::<TileDragEvent>()),
                rotate_tile,
            ),
        )
        // .add_system(spawn_square_on_click)
        .run();
}

fn mouse_button_events(mut mousebtn_evr: EventReader<MouseButtonInput>) {
    use bevy::input::ButtonState;

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

fn mouse_to_world_position(
    // need to get window dimensions
    window: &Query<&Window>,
    // query to get camera transform
    camera_q: &Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, _, camera_transform) = camera_q.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        return world_position;
    }
    eprintln!("Mouse not in window");
    return Vec2::ZERO;
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
            let coords = mouse_to_world_position(&window, &camera_q);

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
                    should_block_lower: false,
                    should_emit_events: true,
                }), // Re-enable picking
                On::<Pointer<Drag>>::send_event::<CubeDragEvent>(),
            ));
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
    if keys.any_pressed([KeyCode::Left]) {
        camera_q.single_mut().1.translation.x -= 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Right]) {
        camera_q.single_mut().1.translation.x += 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Up]) {
        camera_q.single_mut().1.translation.y += 5. * camera_q.single_mut().2.scale;
    } else if keys.any_pressed([KeyCode::Down]) {
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
    if keys.any_pressed([KeyCode::Space]) {
        if camera_q.single_mut().scale > 0.2 {
            camera_q.single_mut().scale -= 0.01;
        }
    } else if keys.any_pressed([KeyCode::ShiftLeft]) {
        if camera_q.single_mut().scale < 3. {
            camera_q.single_mut().scale += 0.01;
        }
    }
}

fn spawn_tile(
    keys: Res<Input<KeyCode>>,
    mut tile_data: ResMut<GameTileData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if keys.just_pressed(KeyCode::T) {
        println!("spawn tile 0");
        create_tile(0, tile_data.into_inner(), commands, meshes, materials);
    }
}

fn rotate_tile(
    keys: Res<Input<KeyCode>>,
    mut raycast: Raycast,
    mut tile_data: ResMut<GameTileData>,
    mut q: Query<(Entity, &mut Transform, &TileInfo), Without<MainCamera>>,
    mut camera_q: Query<(Entity, &mut Camera, &mut Transform), With<MainCamera>>,
) {
    if keys.just_pressed(KeyCode::R) {
        println!("rotate tile");

        let mut ray_pos = camera_q.single_mut().2.translation;
        ray_pos.z += 100.0;
        let ray_dir = Vec3 {
            x: ray_pos.x,
            y: ray_pos.y,
            z: -200.0,
        };
        let ray = Ray3d::new(ray_pos, ray_dir);

        let hits = raycast.cast_ray(ray, &RaycastSettings::default());
        if let Some((ent, _)) = hits.first() {
            let q_tile_info: TileInfo;
            if let Ok((_e, mut _e_transform, tile_info)) = q.get(*ent) {
                q_tile_info = tile_info.clone();
            } else {
                println!("uh oh");
                return;
            }

            for mut tile in q.iter_mut() {
                if tile.2.tile_idx == q_tile_info.tile_idx {
                    let z: f32 = tile.1.translation.z;
                    tile.1.rotate_around(Vec3::new(q_tile_info.area_offset.x, q_tile_info.area_offset.y, z), Quat::from_rotation_z(PI / 2.0))
                    //  todo: fix
                }
            }
        } else {
            println!("No targets found for rotation");
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.init_resource::<GameTileData>();

    commands.spawn((
        Camera2dBundle::default(),
        RaycastPickCamera::default(),
        MainCamera,
    ));

    // // Quad
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes
    //             .add(shape::Quad::new(Vec2::new(50., 100.)).into())
    //             .into(),
    //         material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
    //         transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
    //         ..default()
    //     },
    //     PickableBundle::default(),    // Makes the entity pickable
    //     RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
    //     On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
    //     On::<Pointer<DragEnd>>::target_insert(Pickable), // Re-enable picking
    //     On::<Pointer<Drag>>::send_event::<CubeDragEvent>(),
    //     // On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
    //     //     transform.translation += drag.delta.extend(0.0) // Make the square follow the mouse
    //     // }),
    // ));

    // // Hexagon
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
    //         material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
    //         transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
    //         ..default()
    //     },
    //     PickableBundle::default(),
    //     RaycastPickTarget::default(),
    // ));

    // // Circle overlap with Hexagon
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(shape::Circle::new(25.).into()).into(),
    //         material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //         transform: Transform::from_translation(Vec3::new(150., 0., 1. /* z-height */)),
    //         ..default()
    //     },
    //     PickableBundle::default(),
    //     RaycastPickTarget::default(),
    // ));
}

#[derive(Event)]
pub struct CubeDragEvent(Entity, Vec2);

impl From<ListenerInput<Pointer<Drag>>> for CubeDragEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        CubeDragEvent(event.target, event.delta)
    }
}

pub fn handle_drag_event(
    mut drag_event: EventReader<CubeDragEvent>,
    mut q: Query<(Entity, &mut Transform)>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) {
    for event in drag_event.iter() {
        let e = event.0;
        info!("cube {:?} drag_event {:?}", event.0, event.1);
        if let Ok((_e, mut e_transform)) = q.get_mut(e) {
            let mut translate = event.1.extend(0.0);
            // it's now reversed for some reason, didn't used to be.
            translate.y *= -1.0;
            e_transform.translation += translate * camera_q.single().1.scale;
        } else {
        }
    }
}
