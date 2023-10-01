use std::vec;

use crate::tiles::*;

use bevy::{
    input::mouse::MouseButtonInput, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Event)]
pub struct TileDragEvent(Entity, Vec2);

impl From<ListenerInput<Pointer<Drag>>> for TileDragEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        TileDragEvent(event.target, event.delta)
    }
}

#[derive(Component, Copy, Clone)]
pub struct TileInfo {
    pub tile_idx: TileIndex,
    pub area_idx: TileAreaIndex,
    // x, y
    pub area_offset: Vec2,
}

pub fn handle_tile_area_drag_event(
    mut drag_event: EventReader<TileDragEvent>,
    mut q: Query<(Entity, &mut Transform, &TileInfo)>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) {
    for event in drag_event.iter() {
        let q_tile_info: TileInfo;
        info!("tile {:?} drag_event {:?}", event.0, event.1);
        if let Ok((_e, mut _e_transform, tile_info)) = q.get(event.0) {
            q_tile_info = tile_info.clone();
        } else {
            println!("uh oh");
            return;
        }

        let mut translation: Vec3 = event.1.extend(0.0);
        // it's now reversed for some reason, didn't used to be.
        translation.y *= -1.0;
        translation *= camera_q.single().1.scale;
        for mut ent in q.iter_mut() {
            if ent.2.tile_idx == q_tile_info.tile_idx {
                ent.1.translation += translation;
            }
        }
    }
}

pub fn create_tile(
    tile_id: usize,
    mut tile_data: &GameTileData,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut tile: &Tile = &tile_data.all_tiles[tile_id];

    let off0: Vec2 = Vec2::new(0., 52.5);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(180., 75.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::SEA_GREEN)),
            transform: Transform::from_translation(Vec3::new(off0.x, off0.y, 0.)),
            ..default()
        },
        TileInfo {
            tile_idx: tile_id,
            area_idx: 0,
            area_offset: off0,
        },
        PickableBundle::default(),    // Makes the entity pickable
        RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
        On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable {
            should_block_lower: false,
            should_emit_events: true,
        }), // Re-enable picking
        On::<Pointer<Drag>>::send_event::<TileDragEvent>(),
    ));

    let off1: Vec2 = Vec2::new(0., 0.);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(180., 30.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(off1.x, off1.y, 1.)),
            ..default()
        },
        TileInfo {
            tile_idx: tile_id,
            area_idx: 1,
            area_offset: off1,
        },
        PickableBundle::default(),    // Makes the entity pickable
        RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
        On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable {
            should_block_lower: false,
            should_emit_events: true,
        }), // Re-enable picking
        On::<Pointer<Drag>>::send_event::<TileDragEvent>(),
    ));

    let off2: Vec2 = Vec2::new(0., -52.5);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(180., 75.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::SEA_GREEN)),
            transform: Transform::from_translation(Vec3::new(off2.x, off2.y, 0.)),
            ..default()
        },
        TileInfo {
            tile_idx: tile_id,
            area_idx: 2,
            area_offset: off2,
        },
        PickableBundle::default(),    // Makes the entity pickable
        RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
        On::<Pointer<DragStart>>::target_remove::<Pickable>(), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable {
            should_block_lower: false,
            should_emit_events: true,
        }), // Re-enable picking
        On::<Pointer<Drag>>::send_event::<TileDragEvent>(),
    ));
}
