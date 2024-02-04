use bevy::{
    input::mouse::MouseButtonInput, input::ButtonState, prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_eventlistener::{prelude::*};
use bevy_mod_picking::prelude::*;

use crate::tiles_render::*;

pub fn mouse_button_events(mut mousebtn_evr: EventReader<MouseButtonInput>) {
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

pub fn spawn_square_on_click(
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
