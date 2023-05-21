use std::cell::RefCell;
use std::ptr::null;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Copy, Clone)]
enum AreaType {
    Unspecified,
    Meadow,
    Road,
    RoadTerminus,
    City,
    CityPennant,
    Cloister,
    River,
}

enum TeamColor {
    Blue,
    Red,
}

struct Meeple {
    team: TeamColor,
}

struct TileArea {
    area_type: AreaType,
    connected_nodes: Vec<Rc<RefCell<EdgeNode>>>,
    meeple: Option<Weak<Meeple>>,
}

impl Default for TileArea {
    fn default() -> Self {
        TileArea {
            area_type: AreaType::Unspecified,
            connected_nodes: vec![],
            meeple: None,
        }
    }
}

impl TileArea {
    fn new() -> Self {
        Default::default()
    }
}

struct EdgeNode {
    // Always filled after tiles are created, only an Option for easy construction.
    area: Option<Weak<RefCell<TileArea>>>,
    // Connections to other tiles.
    // Always filled after tiles are created, only an Option for easy construction.
    connections: Option<Weak<EdgeNode>>,
}

impl Default for EdgeNode {
    fn default() -> Self {
        EdgeNode {
            area: None,
            connections: None,
        }
    }
}

impl EdgeNode {
    fn new() -> Self {
        Default::default()
    }
}

struct TileEdge {
    edge_nodes: [RefCell<EdgeNode>; 3],
}

impl Default for TileEdge {
    fn default() -> Self {
        TileEdge {
            edge_nodes: [EdgeNode::new().into(), EdgeNode::new().into(), EdgeNode::new().into()]
        }
    }
}

impl TileEdge {
    fn new() -> Self {
        Default::default()
    }
}

fn get_empty_tile_edges() -> [RefCell<TileEdge>; 4] {
    return [
        TileEdge::new().into(),
        TileEdge::new().into(),
        TileEdge::new().into(),
        TileEdge::new().into(),
    ];
}

fn set_tile_edges_area(edges: &mut [RefCell<TileEdge>; 4], area: &mut TileArea, node_idxs: Vec<usize>) {
    let mut idxs: Vec<[usize; 2]> = vec![];
    for node_idx in node_idxs {
        idxs.push([node_idx % 12, node_idx / 12]);
    }

    for idx in &idxs[..] {
        let w: Weak<EdgeNode> = edges[idx[0]].borrow_mut().edge_nodes[idx[1]].;
        area.connected_nodes.push(w);
    }
    for idx in &idxs[..] {
        // let mut node: &mut EdgeNode = &mut edges[idx[0]].edge_nodes[idx[1]];
        edges[idx[0]].edge_nodes[idx[1]].area = Some(area);
    }
}

struct Tile {
    areas: Vec<RefCell<TileArea>>,
    edges: [RefCell<TileEdge>; 4],
}

fn make_areas(area_types: Vec<AreaType>) -> Vec<TileArea> {
    let mut areas: Vec<TileArea> = vec![];
    for area_type in area_types {
        areas.push(TileArea {
            area_type,
            connected_nodes: vec![],
            meeple: None,
        })
    }
    return areas;
}

// Areas/nodes are ordered clockwise on right edge top node.

fn get_m_r_m5_r_m4() -> Tile {
    let mut areas: Vec<TileArea> = make_areas(vec![AreaType::Meadow, AreaType::Road, AreaType::Meadow]);
    let mut edges: [RefCell<TileEdge>; 4] = get_empty_tile_edges();

    let area0_nodes: Vec<usize> = vec![
        0, 8, 9, 10, 11
    ];
    set_tile_edges_area(&mut edges, &mut areas[0], area0_nodes);

    let area1_nodes: Vec<usize> = vec![
        1, 7
    ];
    set_tile_edges_area(&mut edges, &mut areas[1], area1_nodes);

    let area2_nodes: Vec<usize> = vec![
        2, 3, 4, 5, 6
    ];
    set_tile_edges_area(&mut edges, &mut areas[2], area2_nodes);

    let mut tile: Tile = Tile {
        areas: areas,
        edges: edges
    };

    return tile;
}

fn get_m4_r_m7_c() -> Tile {
    let mut areas: Vec<TileArea> = make_areas(vec![AreaType::Meadow, AreaType::RoadTerminus, AreaType::Cloister]);
    let mut edges: [TileEdge; 4];



    let mut tile: Tile = Tile {
        areas,
        edges,
    };

    return tile;
}

fn get_tiles() -> Vec<Tile> {
    let mut tiles: Vec<Tile>;
    for i in [0..4] {
        tiles.push(get_m_r_m5_r_m4());
    }
    return tiles;
}

struct Player {
    team: TeamColor,
    meeple: Vec<Meeple>,
    points: i32,
}


fn main() {
    let mut tiles: Vec<Tile> = get_tiles();
    println!("Hello, world!");
}
