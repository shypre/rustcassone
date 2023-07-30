mod tiles;

use bevy::{
    input::mouse::MouseButtonInput, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_event::<MouseButtonInput>()
        .add_startup_system(setup)
        .add_event::<CubeDragEvent>()
        .add_systems((mouse_button_events, move_camera, zoom_camera))
        // .add_system(spawn_square_on_click)
        .add_system(handle_drag_event.run_if(on_event::<CubeDragEvent>()))
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

fn my_cursor_system(
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
            let coords = my_cursor_system(&window, &camera_q);

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
                On::<Pointer<DragEnd>>::target_insert(Pickable), // Re-enable picking
                On::<Pointer<Drag>>::send_event::<CubeDragEvent>(),
            ));
        }
    }
}

fn move_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<(&mut Camera, &mut Transform), With<MainCamera>>,
) {
    if keys.any_pressed([KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down]) {
        camera_q.single_mut().1.translation.x += 1.;
    }
}

fn zoom_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_q: Query<&mut OrthographicProjection, (With<Camera>, With<MainCamera>)>,
) {
    if keys.any_pressed([KeyCode::Space]) {
        camera_q.single_mut().scale -= 0.01;
    } else if keys.any_pressed([KeyCode::LShift]) {
        camera_q.single_mut().scale += 0.01;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        RaycastPickCamera::default(),
        MainCamera,
    ));

    // Quad
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(50., 100.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
            transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
            ..default()
        },
        PickableBundle::default(),    // Makes the entity pickable
        RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
        On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable), // Re-enable picking
        On::<Pointer<Drag>>::send_event::<CubeDragEvent>(),
        // On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
        //     transform.translation += drag.delta.extend(0.0) // Make the square follow the mouse
        // }),
    ));

    // Hexagon
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
            material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
            transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
            ..default()
        },
        PickableBundle::default(),
        RaycastPickTarget::default(),
    ));

    // Circle overlap with Hexagon
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(25.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(150., 0., 1. /* z-height */)),
            ..default()
        },
        PickableBundle::default(),
        RaycastPickTarget::default(),
    ));
}

struct CubeDragEvent(Entity, Vec2);

impl From<ListenerInput<Pointer<Drag>>> for CubeDragEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        CubeDragEvent(event.target, event.delta)
    }
}

fn handle_drag_event(
    mut drag_event: EventReader<CubeDragEvent>,
    mut q: Query<(Entity, &mut Transform)>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) {
    for event in drag_event.iter() {
        let e = event.0;
        // info!("cube {:?} drag_event {:?}", event.0, event.1);
        if let Ok((_e, mut e_transform)) = q.get_mut(e) {
            e_transform.translation += event.1.extend(0.0) * camera_q.single().1.scale;
        } else {
        }
    }
}
