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

#[derive(Event)]
pub struct TileDragEvent(Entity, Vec2);

impl From<ListenerInput<Pointer<Drag>>> for TileDragEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        TileDragEvent(event.target, event.delta)
    }
}

pub fn handle_tile_drag_event(
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

#[derive(Event)]
pub struct GenericDragEvent(Entity, Vec2);

impl From<ListenerInput<Pointer<Drag>>> for GenericDragEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        GenericDragEvent(event.target, event.delta)
    }
}

pub fn handle_generic_drag_event(
    mut drag_event: EventReader<GenericDragEvent>,
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
            z_height: -0.1,
        },
        AreaType::Cloister => AreaTypeRenderInfo {
            color: Color::CRIMSON,
            z_height: -0.2,
        },
        AreaType::PennantTown => AreaTypeRenderInfo {
            color: Color::GOLD,
            z_height: -0.3,
        },
        AreaType::Town => AreaTypeRenderInfo {
            color: Color::rgb(0.44, 0.31, 0.22),
            z_height: -0.4,
        },
        AreaType::RoadStopMarker => AreaTypeRenderInfo {
            color: Color::GRAY,
            z_height: -0.5,
        },
        AreaType::EndRoad => AreaTypeRenderInfo {
            color: Color::ANTIQUE_WHITE,
            z_height: -0.6,
        },
        AreaType::Road => AreaTypeRenderInfo {
            color: Color::WHITE,
            z_height: -0.7,
        },
        AreaType::Farm => AreaTypeRenderInfo {
            color: Color::SEA_GREEN,
            z_height: -0.8,
        },
        AreaType::Water => AreaTypeRenderInfo {
            color: Color::BLUE,
            z_height: -0.9,
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
                material: materials.add(ColorMaterial::from(Color::NONE)),
                transform: Transform::from_translation(Vec3::new(
                    mouse_world_pos.x,
                    mouse_world_pos.y,
                    0.0,
                )),
                ..default()
            },
            TileEntityInfo {
                tile_idx,
                area_idxs: tile_data.all_tiles[tile_idx].areas.clone(),
            },
            Highlight {
                // TODO:  put material in resource
                hovered: Some(HighlightKind::Fixed(
                    materials.add(ColorMaterial::from(Color::NONE)),
                )),
                pressed: Some(HighlightKind::Fixed(
                    materials.add(ColorMaterial::from(Color::NONE)),
                )),
                selected: Some(HighlightKind::Fixed(
                    materials.add(ColorMaterial::from(Color::NONE)),
                )),
            },
            parent_pickable_bundle,
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
            area_idx: absolute_area_idx,
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
            ))
            .id();

        commands.entity(parent).push_children(&[child]);
    }
}

pub fn create_FFFF_C() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 180.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(45., 45.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
    ];
    return area_datas;
}

pub fn create_FRFF_C() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 180.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 67.5)).into(),
            offset: Vec2::new(0., -56.25),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(45., 45.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
    ];
    return area_datas;
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

pub fn create_RRRF() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 75.)).into(),
            offset: Vec2::new(0., 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 75.)).into(),
            offset: Vec2::new(0., -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(-52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(-52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 30.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
    ];
    return area_datas;
}

pub fn create_RRRR() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(52.5, 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 75.)).into(),
            offset: Vec2::new(0., -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(-52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(-52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(-52.5, 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 75.)).into(),
            offset: Vec2::new(0., 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 30.)).into(),
            offset: Vec2::new(0., 0.),
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

pub fn create_RFRT_02() -> Vec<AreaRenderDatas> {
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
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_RRFT_01() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: RightTriangleAndTrapezoid::new(180., 75.0).into(),
            offset: Vec2::new(-90., 90.),
            rotation: 3.0 * PI / 2.0,
        },
        AreaRenderDatas {
            mesh: Trapezoid::new(105.0, 30.0).into(),
            offset: Vec2::new(90., -90.),
            rotation: PI / 2.0,
        },
        AreaRenderDatas {
            mesh: RightTriangle::new(75.0).into(),
            offset: Vec2::new(90., -90.),
            rotation: PI / 2.0,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_FRRT_12() -> Vec<AreaRenderDatas> {
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
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_RRRT() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(180., 75.)).into(),
            offset: Vec2::new(0., 52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 75.)).into(),
            offset: Vec2::new(0., -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 75.)).into(),
            offset: Vec2::new(-52.5, -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(75., 30.)).into(),
            offset: Vec2::new(-52.5, 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 30.)).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
    ];
    return area_datas;
}

pub fn create_FTFT() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: SquareWithTwoTrangleChunks::new(180.0).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., -90.0),
            rotation: 0.,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_TFFT() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            // TODO: square for consistency
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(90., 0.),
            rotation: PI / 2.0,
        },
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

pub fn create_TFTF_02() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: SquareWithTwoTrangleChunks::new(180.0).into(),
            offset: Vec2::new(0., 0.),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., -90.0),
            rotation: 0.,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., 90.0),
            rotation: PI,
        },
    ];
    return area_datas;
}

pub fn create_PFPF_02() -> Vec<AreaRenderDatas> {
    return create_TFTF_02();
}

pub fn create_TFFT_03() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: RightTriangle::new(180.0).into(),
            offset: Vec2::new(90., 90.0),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: RightTriangle::new(180.0).into(),
            offset: Vec2::new(-90., -90.0),
            rotation: 0.,
        },
    ];
    return area_datas;
}

pub fn create_PFFP_03() -> Vec<AreaRenderDatas> {
    return create_TFFT_03();
}

pub fn create_TRRT_03_12() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: RightTriangle::new(180.0).into(),
            offset: Vec2::new(90., 90.0),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: Trapezoid::new(180.0, 75.0).into(),
            offset: Vec2::new(-90., -90.0),
            rotation: 0.,
        },
        AreaRenderDatas {
            mesh: Trapezoid::new(105.0, 30.0).into(),
            offset: Vec2::new(-90., -90.0),
            rotation: 0.,
        },
        AreaRenderDatas {
            mesh: RightTriangle::new(75.0).into(),
            offset: Vec2::new(-90., -90.0),
            rotation: 0.,
        },
    ];
    return area_datas;
}

pub fn create_PRRP_03_12() -> Vec<AreaRenderDatas> {
    return create_TRRT_03_12();
}

pub fn create_TFTT_013() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: SquareWithTrangleChunk::new(180.0).into(),
            offset: Vec2::new(0., 0.),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: SquashedTriangle::new(180.0).into(),
            offset: Vec2::new(0., -90.0),
            rotation: 0.,
        },
    ];
    return area_datas;
}

pub fn create_PFPP_013() -> Vec<AreaRenderDatas> {
    return create_TFTT_013();
}

pub fn create_TRTT_013() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![
        AreaRenderDatas {
            mesh: SquareWithTrangleChunk::new(180.0).into(),
            offset: Vec2::new(0., 0.),
            rotation: PI,
        },
        AreaRenderDatas {
            mesh: RightScaleneTriangle::new(45.0, 90.0, false).into(),
            offset: Vec2::new(0., -90.0),
            rotation: 0.,
        },
        AreaRenderDatas {
            mesh: shape::Quad::new(Vec2::new(30., 75.)).into(),
            offset: Vec2::new(0., -52.5),
            rotation: 0.0,
        },
        AreaRenderDatas {
            mesh: RightScaleneTriangle::new(45.0, 90.0, true).into(),
            offset: Vec2::new(0., -90.0),
            rotation: 0.,
        },
        //
    ];
    return area_datas;
}

pub fn create_PRPP_013() -> Vec<AreaRenderDatas> {
    return create_TRTT_013();
}
pub fn create_PPPP_0123() -> Vec<AreaRenderDatas> {
    let area_datas: Vec<AreaRenderDatas> = vec![AreaRenderDatas {
        mesh: shape::Quad::new(Vec2::new(180., 180.)).into(),
        offset: Vec2::new(0., 0.),
        rotation: 0.0,
    }];
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

    let area_data: Vec<AreaRenderDatas>;

    match tile_type {
        TileType::Unspecified => todo!(),
        TileType::RFRF_02 => area_data = create_RFRF_02(),
        TileType::FRRF_12 => area_data = create_FRRF_12(),
        TileType::RRRF => area_data = create_RRRF(),
        TileType::RRRR => area_data = create_RRRR(),
        TileType::FFFF_C => area_data = create_FFFF_C(),
        TileType::FRFF_C => area_data = create_FRFF_C(),
        TileType::FFFT => area_data = create_FFFT(),
        TileType::RFRT_02 => area_data = create_RFRT_02(),
        TileType::RRFT_01 => area_data = create_RRFT_01(),
        TileType::FRRT_12 => area_data = create_FRRT_12(),
        TileType::RRRT => area_data = create_RRRT(),
        TileType::FTFT => area_data = create_FTFT(),
        TileType::TFFT => area_data = create_TFFT(),
        TileType::TFTF_02 => area_data = create_TFTF_02(),
        TileType::PFPF_02 => area_data = create_PFPF_02(),
        TileType::TFFT_03 => area_data = create_TFFT_03(),
        TileType::PFFP_03 => area_data = create_PFFP_03(),
        TileType::TRRT_03_12 => area_data = create_TRRT_03_12(),
        TileType::PRRP_03_12 => area_data = create_PRRP_03_12(),
        TileType::TFTT_013 => area_data = create_TFTT_013(),
        TileType::PFPP_013 => area_data = create_PFPP_013(),
        TileType::TRTT_013 => area_data = create_TRTT_013(),
        TileType::PRPP_013 => area_data = create_PRPP_013(),
        TileType::PPPP_0123 => area_data = create_PPPP_0123(),
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
pub fn create_placeholder_tile(
    window: &Window,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let mouse_world_pos: Vec2 = mouse_to_world_position(window, camera, camera_transform);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(180., 180.)).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::BEIGE)),
            transform: Transform::from_translation(Vec3::new(
                mouse_world_pos.x,
                mouse_world_pos.y,
                -10.0,
            )),
            ..default()
        },
        Highlight {
            // TODO:  put material in resource
            hovered: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::BEIGE)),
            )),
            pressed: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::BEIGE)),
            )),
            selected: Some(HighlightKind::Fixed(
                materials.add(ColorMaterial::from(Color::BEIGE)),
            )),
        },
        PickableBundle::default(),
        RaycastPickTarget::default(), // Marker for the `bevy_picking_raycast` backend
        On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
        On::<Pointer<Drag>>::send_event::<GenericDragEvent>(),
        On::<Pointer<Drop>>::commands_mut(|event, commands| {
            println!("dropped");
        }),
    ));
}
