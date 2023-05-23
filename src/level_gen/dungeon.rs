use bevy::prelude::{IVec2, UVec2};

struct Bounds {
    pos: IVec2,
    extents: UVec2,
}

struct Room {
    bounds: Bounds,
}

struct Dungeon {
    bounds: Bounds,
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Into<IVec2> for Direction {
    fn into(self) -> IVec2 {
        match self {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }.into()
    }
}

struct Corridor {
    pos: IVec2,
    len: u32,
    dir: Direction,
}


fn generate_dungeon() {

}

