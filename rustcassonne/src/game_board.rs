use crate::myshapes::*;
use crate::tiles::*;

use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseButtonInput, input::ButtonState, prelude::*, render::camera::RenderTarget,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_eventlistener::{callbacks::ListenerInput, prelude::*};
use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::{
    system_param::{Raycast, RaycastSettings},
    *,
};
use petgraph::{
    data::FromElements,
    dot::{Config, Dot},
    stable_graph::{NodeIndex, StableGraph},
    Undirected,
};
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileDirection {
    UP,
    RIGHT,
    DOWN,
    LEFT,
    NONE,
}

pub fn direction_to_offset(dir: TileDirection) -> Vec2 {
    match dir {
        TileDirection::UP => Vec2 { x: 0.0, y: 180.0 },
        TileDirection::RIGHT => Vec2 { x: 180.0, y: 0.0 },
        TileDirection::DOWN => Vec2 { x: 0.0, y: -180.0 },
        TileDirection::LEFT => Vec2 { x: -180.0, y: 0.0 },
        TileDirection::NONE => Vec2 { x: 0.0, y: 0.0 },
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct GameplayData {
    pub spawned_tiles: Vec<TileIndex>,
    pub unspawned_tiles: Vec<TileIndex>,
    pub board_tile_graph: StableGraph<TileIndex, TileDirection, Undirected>,
    pub tile_index_to_tile_graph_index: HashMap<TileIndex, NodeIndex>,
    pub next_placeholder_index: TileIndex,
    pub board_area_graph: StableGraph<TileAreaIndex, (), Undirected>,
    pub area_index_to_area_graph_index: HashMap<TileAreaIndex, NodeIndex>,
}

impl GameplayData {
    pub fn print(&self) {
        println!("spawned_tiles: {:?}", self.spawned_tiles);
        println!("unspawned_tiles: {:?}", self.unspawned_tiles);
        println!("next_placeholder_index: {:?}", self.next_placeholder_index);
        println!(
            "tile_index_to_tile_graph_index: {:?}",
            self.tile_index_to_tile_graph_index
        );
        println!(
            "area_index_to_area_graph_index: {:?}",
            self.area_index_to_area_graph_index
        );
        println!(
            "board_tile_graph:\n{:?}",
            Dot::with_config(&self.board_tile_graph, &[])
        );
        println!(
            "board_area_graph:\n{:?}",
            Dot::with_config(&self.board_area_graph, &[])
        );
    }
}
