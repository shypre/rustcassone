use std::f32::consts::PI;
use std::vec;

use crate::myshapes::*;
use crate::tiles::*;

use bevy::{
    input::mouse::MouseButtonInput, math::vec4, prelude::*, render::camera::RenderTarget,
    render::mesh::Mesh, sprite::MaterialMesh2dBundle, window::PrimaryWindow,
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
pub struct AreaEntityInfo {
    pub tile_idx: TileIndex,
    pub area_idx: TileAreaIndex,
    // x, y
    pub area_offset: Vec2,
}

#[derive(Component, Clone)]
pub struct TileEntityInfo {
    pub tile_idx: TileIndex,
    pub area_idxs: Vec<TileAreaIndex>,
}

pub fn handle_tile_area_drag_event(
    mut drag_event: EventReader<TileDragEvent>,
    mut q: Query<(Entity, &mut Transform, &TileEntityInfo)>,
    camera_q: Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) {
    for event in drag_event.iter() {
        let q_tile_info: TileEntityInfo;
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

pub fn mouse_to_world_position(
    // the window that the camera is displaying to (or the primary window)
    window: &Window,
    // camera component and transform
    camera_component: &Camera,
    camera_transform: &GlobalTransform,
    // camera_q: &Query<(&Camera, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
) -> Vec2 {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    // let (camera, _, camera_transform) = camera_q.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera_component.viewport_to_world_2d(camera_transform, cursor))
    {
        eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        return world_position;
    }
    eprintln!("Mouse not in window");
    return Vec2::ZERO;
}

pub struct AreaTypeRenderInfo {
    color: Color,
    z_height: f32,
}

pub fn get_area_type_info(area_type: AreaType) -> AreaTypeRenderInfo {
    match area_type {
        AreaType::Unspecified => AreaTypeRenderInfo {
            color: Color::ORANGE,
            z_height: -1.0,
        },
        AreaType::Farm => AreaTypeRenderInfo {
            color: Color::SEA_GREEN,
            z_height: -2.0,
        },
        AreaType::Road => AreaTypeRenderInfo {
            color: Color::WHITE,
            z_height: -3.0,
        },
        AreaType::EndRoad => AreaTypeRenderInfo {
            color: Color::ANTIQUE_WHITE,
            z_height: -4.0,
        },
        AreaType::Town => AreaTypeRenderInfo {
            color: Color::rgb(0.44, 0.31, 0.22),
            z_height: -5.0,
        },
        AreaType::PennantTown => AreaTypeRenderInfo {
            color: Color::GOLD,
            z_height: -6.0,
        },
        AreaType::Cloister => AreaTypeRenderInfo {
            color: Color::CRIMSON,
            z_height: -7.0,
        },
        AreaType::Water => AreaTypeRenderInfo {
            color: Color::BLUE,
            z_height: -8.0,
        },
    }
}

pub fn get_absolute_area_from_relative_area(
    tile_idx: TileIndex,
    relative_area_idx: TileAreaIndex,
    tile_data: &GameTileData,
) -> TileAreaIndex {
    return tile_data.all_tiles[tile_idx].areas[relative_area_idx];
}

pub struct AreaRenderDatas {
    pub mesh: Mesh,
    pub offset: Vec2,
    pub rotation: f32,
}

pub fn create_areas(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tile_data: &GameTileData,
    tile_idx: TileIndex,
    mouse_world_pos: Vec2,
    area_render_datas: &Vec<AreaRenderDatas>, /* mesh, area_offset */
) {
    let mut relative_area_idx = 0;

    // check we have provided enough areas for this tile
    let num_areas = tile_data.all_tiles[tile_idx].areas.len();
    assert!(num_areas == area_render_datas.len());

    let mut parent_pickable_bundle = PickableBundle::default();
    parent_pickable_bundle.pickable = Pickable {
        should_block_lower: true,
        should_emit_events: true,
    };

    let parent = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(180., 180.)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 0.0,
                })),
                transform: Transform::from_translation(Vec3::new(
                    mouse_world_pos.x,
                    mouse_world_pos.y,
                    10.,
                )),
                ..default()
            },
            TileEntityInfo {
                tile_idx,
                area_idxs: tile_data.all_tiles[tile_idx].areas.clone(),
            },
            parent_pickable_bundle,
            Highlight {
                // TODO:  put material in resource
                hovered: Some(HighlightKind::Fixed(materials.add(ColorMaterial {
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    texture: None,
                }))),
                pressed: Some(HighlightKind::Fixed(materials.add(ColorMaterial {
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    texture: None,
                }))),
                selected: Some(HighlightKind::Fixed(materials.add(ColorMaterial {
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    texture: None,
                }))),
            },
            RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
            On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
            On::<Pointer<DragEnd>>::target_insert(Pickable {
                should_block_lower: true,
                should_emit_events: true,
            }), // Re-enable picking
            On::<Pointer<Drag>>::send_event::<TileDragEvent>(),
        ))
        .id();

    for area_data in area_render_datas {
        let mesh = &area_data.mesh;
        let area_offset = &area_data.offset;
        let area_rotation = area_data.rotation;

        let absolute_area_idx =
            get_absolute_area_from_relative_area(tile_idx, relative_area_idx, tile_data);
        relative_area_idx += 1;

        let area_info = AreaEntityInfo {
            tile_idx,
            area_idx: absolute_area_idx, // TODO: use absolute index
            area_offset: *area_offset,
        };

        let area_type_info: AreaTypeRenderInfo =
            get_area_type_info(tile_data.all_areas[absolute_area_idx].area_type);
        let mut transform = Transform::from_translation(Vec3::new(
            area_offset.x,
            area_offset.y,
            area_type_info.z_height,
        ));
        transform.rotate_z(area_rotation);
        let child = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(mesh.clone()).into(),
                    material: materials.add(ColorMaterial::from(area_type_info.color)),
                    transform: transform,
                    ..default()
                },
                area_info,
                PickableBundle::default(), // Makes the entity pickable
                RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
                                           // On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                                           // On::<Pointer<DragEnd>>::target_insert(Pickable {
                                           //     should_block_lower: false,
                                           //     should_emit_events: true,
                                           // }), // Re-enable picking
                                           // On::<Pointer<Drag>>::send_event::<TileDragEvent>(),
            ))
            .id();

        commands.entity(parent).push_children(&[child]);
    }
}

pub fn create_RFRF_02() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 75.)).into(),
            offset: Vec2::new(0., 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 30.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 75.)).into(),
            offset: Vec2::new(0., -52.5),
            rotation: 0.0,
        },
    ];
    return area_datas;
}

pub fn create_FRRF_12() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: RightTriangleAndTrapezoid::new(180., 75.0).into(),
            offset: Vec2::new(90., 90.),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: Trapezoid::new(105.0, 30.0).into(),
            offset: Vec2::new(-90., -90.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: RightTriangle::new(75.0).into(),
            offset: Vec2::new(-90., -90.),
            rotation: 0.0,
        },
    ];
    return area_datas;
}

pub fn create_FFFT() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: SquareWithTrangleChunk::new(180.0).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_tile(
    tile_idx: TileIndex,
    window: &Window,
    mut tile_data: &GameTileData,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    // let mut tile: &Tile = &tile_data.all_tiles[tile_id];

    let mouse_world_pos: Vec2 = mouse_to_world_position(window, camera, camera_transform);

    let tile_type: TileType = tile_data.all_tiles[tile_idx].tile_type;

    let mut area_data: Vec<AreaRenderDatas>;

    match tile_type {
        TileType::Unspecified => todo!(),
        TileType::RFRF_02 => area_data = create_RFRF_02(),
        TileType::FRRF_12 => area_data = create_FRRF_12(),
        TileType::RRRF => todo!(),
        TileType::RRRR => todo!(),
        TileType::FFFF_C => todo!(),
        TileType::FRFF_C => todo!(),
        TileType::FFFT => area_data = create_FFFT(),
        TileType::RFRT_02 => todo!(),
        TileType::RRFT_01 => todo!(),
        TileType::FRRT_12 => todo!(),
        TileType::RRRT => todo!(),
        TileType::FTFT => todo!(),
        TileType::TFFT => todo!(),
        TileType::TFTF_02 => todo!(),
        TileType::PFPF_02 => todo!(),
        TileType::TFFT_03 => todo!(),
        TileType::PFFP_03 => todo!(),
        TileType::TRRT_03_12 => todo!(),
        TileType::PRRP_03_12 => todo!(),
        TileType::TFTT_013 => todo!(),
        TileType::PFPP_013 => todo!(),
        TileType::TRTT_013 => todo!(),
        TileType::PRPP_013 => todo!(),
        TileType::PPPP_0123 => todo!(),
        TileType::FWFF => todo!(),
        TileType::WFWF_02 => todo!(),
        TileType::FWWF_12 => todo!(),
        TileType::WRWF_02_C => todo!(),
        TileType::WRWR_02_13 => todo!(),
        TileType::RWWR_03_12 => todo!(),
        TileType::WRWT_02 => todo!(),
        TileType::WTWT_02 => todo!(),
        TileType::TWWT_03 => todo!(),
    }

    create_areas(
        commands,
        meshes,
        materials,
        tile_data,
        tile_idx,
        mouse_world_pos,
        &area_data,
    );
}
