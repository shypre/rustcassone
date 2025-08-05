pub type MeepleIndex = usize;

pub enum TeamColor {
    Blue,
    Red,
}

pub struct Meeple {
    team: TeamColor,
}

pub struct Player {
    pub team: TeamColor,
    pub meeples: Vec<MeepleIndex>,
    pub points: i32,
}

pub const NUM_MEEPLES: i32 = 7;
