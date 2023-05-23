use super::{matrix::Matrix, point::Point};

const DEFAULT_TILE_DENSITY: i8 = 0;

/// square tilemap which returns a default density for
/// indices outside it's range.
pub struct Tiles {
    densities : Matrix<i8, 2>,
    dist_between_nodes: f64
}

impl Tiles {
    pub fn new(densities : Matrix<i8, 2>, dist_between_nodes : f64) -> Self {
        Self {
            densities,
            dist_between_nodes
        }
    }

    pub fn get(&self, loc: Point<i32>) -> i8 {
        match [loc.x, loc.y].map(|x| usize::try_from(x)) {
            [Ok(x), Ok(y)] if x < self.densities.dim()[0] && y < self.densities.dim()[1] => {
                self.densities.get([x, y])
            }
            _ => DEFAULT_TILE_DENSITY,
        }
    }

    pub fn dimension(&self) -> [usize; 2] {
        self.densities.dim()
    }

    pub fn dist_between_nodes(&self) -> f64 {
        self.dist_between_nodes
    }
}
