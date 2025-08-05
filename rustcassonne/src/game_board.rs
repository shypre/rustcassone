use crate::tiles::*;

use bevy::prelude::*;
use petgraph::{
    dot::Dot,
    stable_graph::{NodeIndex, StableGraph},
    Undirected,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TileMatrixCoords {
    pub x: i32,
    pub y: i32,
}

// Starts from north and goes clockwise.
pub const NEIGHBOR_COORDS: [TileMatrixCoords; 4] = [
    TileMatrixCoords { x: 0, y: 1 },
    TileMatrixCoords { x: 1, y: 0 },
    TileMatrixCoords { x: 0, y: -1 },
    TileMatrixCoords { x: -1, y: 0 },
];

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct TileMatrixInfo {
//     pub tile_index: TileIndex,
// }

#[derive(Resource, Default, Clone, Debug)]
pub struct GameplayData {
    pub spawned_tiles: Vec<TileIndex>,
    pub unspawned_tiles: Vec<TileIndex>,
    // pub board_tile_graph: StableGraph<TileIndex, TileDirection, Undirected>,
    pub board_tile_matrix: HashMap<TileMatrixCoords, TileIndex>,
    pub board_tile_matrix_inverse: HashMap<TileIndex, TileMatrixCoords>,
    // pub tile_index_to_tile_graph_index: HashMap<TileIndex, NodeIndex>,
    pub next_placeholder_index: TileIndex,
    pub board_area_graph: StableGraph<TileAreaIndex, (), Undirected>,
    pub area_index_to_area_graph_index: HashMap<TileAreaIndex, NodeIndex>,
}

impl GameplayData {
    pub fn print(&self) {
        println!("spawned_tiles: {:?}", self.spawned_tiles);
        println!("unspawned_tiles: {:?}", self.unspawned_tiles);
        println!("next_placeholder_index: {:?}", self.next_placeholder_index);
        // println!(
        //     "tile_index_to_tile_graph_index: {:?}",
        //     self.tile_index_to_tile_graph_index
        // );
        println!("board_tile_matrix: {:?}", self.board_tile_matrix);
        println!(
            "board_tile_matrix_inverse: {:?}",
            self.board_tile_matrix_inverse
        );
        println!(
            "area_index_to_area_graph_index: {:?}",
            self.area_index_to_area_graph_index
        );
        // println!(
        //     "board_tile_graph:\n{:?}",
        //     Dot::with_config(&self.board_tile_matrix, &[])
        // );
        println!(
            "board_area_graph:\n{:?}",
            Dot::with_config(&self.board_area_graph, &[])
        );
    }
}
