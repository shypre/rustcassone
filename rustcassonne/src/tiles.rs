use bevy::prelude::Resource;

#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum AreaType {
    Unspecified,
    Farm,
    Road,
    EndRoad,
    Town,
    PennantTown,
    Cloister,
    Water,
}

#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum TileType {
    Unspecified,
    FRF_FFF_FRF_FFF,
    FFF_FRF_FRF_FFF,
}

pub enum TeamColor {
    Blue,
    Red,
}

pub struct Meeple {
    team: TeamColor,
}

pub type EdgeNumber = usize;
pub type TileAreaIndex = usize;
pub type MeepleIndex = usize;
pub type TileIndex = usize;

#[derive(Debug)]
#[derive(Clone)]
pub struct TileArea {
    pub area_type: AreaType,
    pub self_idx: TileAreaIndex,
    pub edges: Vec<EdgeNumber>,
    // To areas in same tile, only used for meadow-city interactions.
    pub connected_areas: Vec<TileAreaIndex>,
}

fn create_area(area_type: AreaType, edges: Vec<EdgeNumber>) -> TileArea {
    return TileArea {
        area_type,
        self_idx: 0,
        edges,
        connected_areas: vec![],
    };
}

fn fill_area_idxs(areas: &mut Vec<TileArea>, mut offset: TileAreaIndex) -> Vec<TileAreaIndex> {
    let mut idxs: Vec<TileAreaIndex> = vec![];
    for area in areas {
        area.self_idx = offset;
        idxs.push(offset);
        offset += 1;
    }
    return idxs;
}

fn make_tile_area_connections(
    areas: &mut Vec<TileArea>,
    conns: Vec<[TileAreaIndex; 2]>,
    offset: TileAreaIndex,
) {
    for conn in conns {
        areas[offset + conn[0]]
            .connected_areas
            .push(offset + conn[1]);
        areas[offset + conn[1]]
            .connected_areas
            .push(offset + conn[0]);
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Tile {
    pub areas: Vec<TileAreaIndex>,
    pub tile_type: TileType,
}

// Indices start from 0 on right edge top node and go clockwise based on
// https://en.wikipedia.org/wiki/Carcassonne_(board_game)#Tiles

fn get_frf_fff_frf_fff(all_areas: &mut Vec<TileArea>) -> Tile {
    let offs: TileAreaIndex = all_areas.len();

    let mut areas: Vec<TileArea> = vec![
        create_area(AreaType::Farm, vec![0, 8, 9, 10, 11]),
        create_area(AreaType::Road, vec![1, 7]),
        create_area(AreaType::Farm, vec![2, 3, 4, 5, 6]),
    ];
    let idxs: Vec<TileAreaIndex> = fill_area_idxs(&mut areas, offs);

    all_areas.append(&mut areas);
    return Tile {
        areas: idxs,
        tile_type: TileType::FRF_FFF_FRF_FFF,
    };
}

fn get_fff_frf_frf_fff(all_areas: &mut Vec<TileArea>) -> Tile {
    let offs: TileAreaIndex = all_areas.len();

    let mut areas: Vec<TileArea> = vec![
        create_area(AreaType::Farm, vec![0, 1, 2, 3, 8, 9, 10, 11]),
        create_area(AreaType::Road, vec![4, 7]),
        create_area(AreaType::Farm, vec![5, 6]),
    ];
    let idxs: Vec<TileAreaIndex> = fill_area_idxs(&mut areas, offs);

    all_areas.append(&mut areas);
    return Tile {
        areas: idxs,
        tile_type: TileType::FFF_FRF_FRF_FFF,
    };
}

#[derive(Resource)]
#[derive(Clone)]
#[derive(Debug)]
pub struct GameTileData {
    pub all_areas: Vec<TileArea>,
    pub all_tiles: Vec<Tile>,
}

impl Default for GameTileData {
    fn default() -> Self {
        return create_tiles();
    }
}

pub fn create_tiles() -> GameTileData {
    let mut game_tiles = GameTileData {
        all_areas: vec![],
        all_tiles: vec![],
    };
    for _i in 0..8 {
        game_tiles
            .all_tiles
            .push(get_frf_fff_frf_fff(&mut game_tiles.all_areas));
    }
    for _i in 0..9 {
        game_tiles
            .all_tiles
            .push(get_fff_frf_frf_fff(&mut game_tiles.all_areas));
    }
    return game_tiles;
}

pub struct Player {
    pub team: TeamColor,
    pub meeples: Vec<MeepleIndex>,
    pub points: i32,
}

// All areas connected across tiles.
pub struct MegaArea {
    pub connected_areas: Vec<TileAreaIndex>,
}
