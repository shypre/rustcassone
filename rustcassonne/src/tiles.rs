#[derive(Copy, Clone)]
pub enum AreaType {
    Unspecified,
    Meadow,
    Road,
    RoadTerminus,
    City,
    CityPennant,
    Cloister,
    River,
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

pub struct TileArea {
    area_type: AreaType,
    self_idx: TileAreaIndex,
    edges: Vec<EdgeNumber>,
    // To areas in same tile, only used for meadow-city interactions.
    connected_areas: Vec<TileAreaIndex>,
}

fn create_area(area_type: AreaType, edges: Vec<EdgeNumber>) -> TileArea {
    return TileArea {
        area_type,
        self_idx: 0,
        edges,
        connected_areas: vec![],
    }
}

fn fill_area_idxs(areas: &mut Vec<TileArea>, offset: TileAreaIndex) -> Vec<TileAreaIndex> {
    let mut i: TileAreaIndex = 0;
    let mut idxs: Vec<TileAreaIndex> = vec![];
    for area in areas {
        area.self_idx = offset + i;
        idxs.push(offset + i);
        i += 1;
    }
    return idxs;
}

fn make_tile_area_connections(areas: &mut Vec<TileArea>, conns: Vec<[TileAreaIndex; 2]>, offset: TileAreaIndex) {
    for conn in conns {
        areas[offset + conn[0]].connected_areas.push(offset + conn[1]);
        areas[offset + conn[1]].connected_areas.push(offset + conn[0]);
    }
}


pub struct Tile {
    areas: Vec<TileAreaIndex>,
}

// Indices start from 0 on right edge top node and go clockwise based on
// https://en.wikipedia.org/wiki/Carcassonne_(board_game)#Tiles

fn get_mrm_mmm_mrm_mmm(all_areas: &mut Vec<TileArea>) -> Tile {
    let offs: TileAreaIndex = all_areas.len();

    let mut areas: Vec<TileArea> = vec![
        create_area(AreaType::Meadow, vec![0, 8, 9, 10, 11]),
        create_area(AreaType::Road, vec![1, 7]),
        create_area(AreaType::Meadow, vec![2, 3, 4, 5, 6]),
    ];
    let idxs: Vec<TileAreaIndex> = fill_area_idxs(&mut areas, offs);
    
    all_areas.append(&mut areas);
    return Tile {
        areas: idxs
    };
}

fn get_mmm_mrm_mrm_mmm(all_areas: &mut Vec<TileArea>) -> Tile {
    let offs: TileAreaIndex = all_areas.len();

    let mut areas: Vec<TileArea> = vec![
        create_area(AreaType::Meadow, vec![0, 1, 2, 3, 8, 9, 10, 11]),
        create_area(AreaType::Road, vec![4, 7]),
        create_area(AreaType::Meadow, vec![5, 6]),
    ];
    let idxs: Vec<TileAreaIndex> = fill_area_idxs(&mut areas, offs);
    
    all_areas.append(&mut areas);
    return Tile {
        areas: idxs
    };
}

pub fn create_tiles(all_areas: &mut Vec<TileArea>, all_tiles: &mut Vec<Tile>) {
    for _i in [0..8] {
        all_tiles.push(get_mrm_mmm_mrm_mmm(all_areas));
    }
    for _i in [0..9] {
        all_tiles.push(get_mmm_mrm_mrm_mmm(all_areas));
    }
}

pub struct Player {
    team: TeamColor,
    meeples: Vec<MeepleIndex>,
    points: i32,
}

// All areas connected across tiles.
pub struct MegaArea {
    connected_areas: Vec<TileAreaIndex>,
}
