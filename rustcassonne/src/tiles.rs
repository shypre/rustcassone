use bevy::prelude::Resource;

#[derive(Copy, Clone, Debug)]
pub enum AreaType {
    Unspecified,
    Farm,
    Road,
    EndRoad,
    RoadStopMarker,
    Town,
    PennantTown,
    Cloister,
    Water,
}

// Starting from right edge. Each group after first 4 letters is connected
// road or town edges, and last optional groups is if cloister is present.
// https://en.wikipedia.org/wiki/Carcassonne_(board_game)#Tiles
#[derive(Copy, Clone, Debug)]
pub enum TileType {
    Unspecified,
    // base game
    RFRF_02,
    FRRF_12,
    RRRF,
    RRRR,
    FFFF_C,
    FRFF_C,
    //
    FFFT,
    RFRT_02,
    RRFT_01,
    FRRT_12,
    RRRT,
    //
    FTFT,
    TFFT,
    TFTF_02,
    PFPF_02,
    TFFT_03,
    PFFP_03,
    TRRT_03_12,
    PRRP_03_12,
    //
    TFTT_013,
    PFPP_013,
    TRTT_013,
    PRPP_013,
    //
    PPPP_0123,
    // waters
    FWFF,
    WFWF_02,
    FWWF_12,
    WRWF_02_C,
    WRWR_02_13,
    RWWR_03_12,
    //
    WRWT_02,
    //
    WTWT_02,
    TWWT_03,
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

#[derive(Debug, Clone)]
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
    let mut all_edges: Vec<EdgeNumber> = vec![];
    for a in &mut *areas {
        all_edges.extend(&a.edges);
    }
    all_edges.sort();
    if all_edges != vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] {
        println!(
            "all_edges did not contain all edges of tile. areas: {:?}",
            areas
        );
        assert!(false);
    }

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

#[derive(Debug, Clone)]
pub struct Tile {
    pub areas: Vec<TileAreaIndex>,
    pub tile_type: TileType,
}

fn get_tile(tile_type: TileType, all_areas: &mut Vec<TileArea>) -> Tile {
    let offs: TileAreaIndex = all_areas.len();

    let mut areas: Vec<TileArea>;
    match tile_type {
        TileType::RFRF_02 => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 8, 9, 10, 11]),
                create_area(AreaType::Road, vec![1, 7]),
                create_area(AreaType::Farm, vec![2, 3, 4, 5, 6]),
            ]
        }
        TileType::FRRF_12 => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 3, 8, 9, 10, 11]),
                create_area(AreaType::Road, vec![4, 7]),
                create_area(AreaType::Farm, vec![5, 6]),
            ];
        }
        //
        TileType::RRRF => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 8, 9, 10, 11]),
                create_area(AreaType::EndRoad, vec![1]),
                create_area(AreaType::Farm, vec![2, 3]),
                create_area(AreaType::EndRoad, vec![4]),
                create_area(AreaType::Farm, vec![5, 6]),
                create_area(AreaType::EndRoad, vec![7]),
                create_area(AreaType::RoadStopMarker, vec![]),
            ];
        }
        TileType::RRRR => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 11]),
                create_area(AreaType::EndRoad, vec![1]),
                create_area(AreaType::Farm, vec![2, 3]),
                create_area(AreaType::EndRoad, vec![4]),
                create_area(AreaType::Farm, vec![5, 6]),
                create_area(AreaType::EndRoad, vec![7]),
                create_area(AreaType::Farm, vec![8, 9]),
                create_area(AreaType::EndRoad, vec![10]),
                create_area(AreaType::RoadStopMarker, vec![]),
            ];
        }
        TileType::FFFF_C => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
                //
                create_area(AreaType::Cloister, vec![]),
            ];
        }
        TileType::FRFF_C => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11]),
                create_area(AreaType::EndRoad, vec![4]),
                //
                create_area(AreaType::Cloister, vec![]),
            ];
        }
        TileType::FFFT => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::RFRT_02 => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 8]),
                create_area(AreaType::Road, vec![1, 7]),
                create_area(AreaType::Farm, vec![2, 3, 4, 5, 6]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::RRFT_01 => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 5, 6, 7, 8]),
                create_area(AreaType::Road, vec![1, 4]),
                create_area(AreaType::Farm, vec![2, 3]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::FRRT_12 => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 3, 8]),
                create_area(AreaType::Road, vec![4, 7]),
                create_area(AreaType::Farm, vec![5, 6]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::RRRT => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 8]),
                create_area(AreaType::EndRoad, vec![1]),
                create_area(AreaType::Farm, vec![2, 3]),
                create_area(AreaType::EndRoad, vec![4]),
                create_area(AreaType::Farm, vec![5, 6]),
                create_area(AreaType::EndRoad, vec![7]),
                create_area(AreaType::Town, vec![9, 10, 11]),
                create_area(AreaType::RoadStopMarker, vec![]),
            ];
        }
        TileType::FTFT => {
            areas = vec![
                create_area(AreaType::Farm, vec![0, 1, 2, 6, 7, 8]),
                create_area(AreaType::Town, vec![3, 4, 5]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::TFFT => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2]),
                create_area(AreaType::Farm, vec![3, 4, 5, 6, 7, 8]),
                create_area(AreaType::Town, vec![9, 10, 11]),
            ];
        }
        TileType::TFTF_02 => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2, 6, 7, 8]),
                create_area(AreaType::Farm, vec![3, 4, 5]),
                create_area(AreaType::Farm, vec![9, 10, 11]),
            ];
        }
        TileType::PFPF_02 => {
            areas = vec![
                create_area(AreaType::PennantTown, vec![0, 1, 2, 6, 7, 8]),
                create_area(AreaType::Farm, vec![3, 4, 5]),
                create_area(AreaType::Farm, vec![9, 10, 11]),
            ];
        }
        TileType::TFFT_03 => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 4, 5, 6, 7, 8]),
            ];
        }
        TileType::PFFP_03 => {
            areas = vec![
                create_area(AreaType::PennantTown, vec![0, 1, 2, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 4, 5, 6, 7, 8]),
            ];
        }
        TileType::TRRT_03_12 => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 8]),
                create_area(AreaType::Road, vec![4, 7]),
                create_area(AreaType::Farm, vec![5, 6]),
            ];
        }
        TileType::PRRP_03_12 => {
            areas = vec![
                create_area(AreaType::PennantTown, vec![0, 1, 2, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 8]),
                create_area(AreaType::Road, vec![4, 7]),
                create_area(AreaType::Farm, vec![5, 6]),
            ];
        }
        TileType::TFTT_013 => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2, 6, 7, 8, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 4, 5]),
            ];
        }
        TileType::PFPP_013 => {
            areas = vec![
                create_area(AreaType::PennantTown, vec![0, 1, 2, 6, 7, 8, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3, 4, 5]),
            ];
        }
        TileType::TRTT_013 => {
            areas = vec![
                create_area(AreaType::Town, vec![0, 1, 2, 6, 7, 8, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3]),
                create_area(AreaType::EndRoad, vec![4]),
                create_area(AreaType::Farm, vec![5]),
            ];
        }
        TileType::PRPP_013 => {
            areas = vec![
                create_area(AreaType::PennantTown, vec![0, 1, 2, 6, 7, 8, 9, 10, 11]),
                create_area(AreaType::Farm, vec![3]),
                create_area(AreaType::EndRoad, vec![4]),
                create_area(AreaType::Farm, vec![5]),
            ];
        }
        TileType::PPPP_0123 => {
            areas = vec![create_area(
                AreaType::PennantTown,
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            )];
        }
        //
        TileType::FWFF => todo!(),
        TileType::WFWF_02 => todo!(),
        TileType::FWWF_12 => todo!(),
        TileType::WRWF_02_C => todo!(),
        TileType::WRWR_02_13 => todo!(),
        TileType::RWWR_03_12 => todo!(),
        TileType::WRWT_02 => todo!(),
        TileType::WTWT_02 => todo!(),
        TileType::TWWT_03 => todo!(),
        TileType::Unspecified => todo!(),
    }
    let idxs: Vec<TileAreaIndex> = fill_area_idxs(&mut areas, offs);

    all_areas.append(&mut areas);
    return Tile {
        areas: idxs,
        tile_type,
    };
}

#[derive(Resource, Clone, Debug)]
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

    let tiles_and_qty: Vec<(TileType, usize)> = vec![
        (TileType::RFRF_02, 8),
        (TileType::FRRF_12, 9),
        (TileType::RRRF, 4),
        (TileType::RRRR, 1),
        (TileType::FFFF_C, 4),
        (TileType::FRFF_C, 2),
        (TileType::FFFT, 5),
        (TileType::RFRT_02, 4),
        (TileType::RRFT_01, 3),
        (TileType::FRRT_12, 3),
        (TileType::RRRT, 3),
        (TileType::FTFT, 3),
        (TileType::TFFT, 2),
        (TileType::TFTF_02, 1),
        (TileType::PFPF_02, 2),
        (TileType::TFFT_03, 3),
        (TileType::PFFP_03, 2),
        (TileType::TRRT_03_12, 3),
        (TileType::PRRP_03_12, 2),
        (TileType::TFTT_013, 3),
        (TileType::PFPP_013, 1),
        (TileType::TRTT_013, 1),
        (TileType::PRPP_013, 2),
        (TileType::PPPP_0123, 1),
    ];

    for (tile_type, qty) in tiles_and_qty {
        for _i in 0..qty {
            game_tiles
                .all_tiles
                .push(get_tile(tile_type, &mut game_tiles.all_areas));
        }
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
