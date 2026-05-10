use rand::Rng;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const GRID_SIZE: usize = 65;

#[derive(Copy, Clone, Eq, PartialEq)]
struct PathState {
    cost: usize,
    position: (usize, usize),
}

impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Environment {
    pub grid: [[bool; GRID_SIZE]; GRID_SIZE],
    pub hotspots: [(usize, usize); 4],
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            grid: [[false; GRID_SIZE]; GRID_SIZE],
            hotspots: [(2, 2), (27, 2), (2, 27), (27, 27)],
        };

        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if x % 5 == 0 || y % 5 == 0 {
                    env.grid[x][y] = true;
                }
            }
        }
        env
    }

    pub fn get_random_building_cell(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..GRID_SIZE);
            let y = rng.gen_range(0..GRID_SIZE);
            if !self.grid[x][y] {
                return (x, y);
            }
        }
    }

    pub fn calculate_path(
        &self,
        start: (usize, usize),
        goal: (usize, usize),
    ) -> Vec<(usize, usize)> {
        if start == goal {
            return Vec::new();
        }

        let mut dist = vec![vec![usize::MAX; GRID_SIZE]; GRID_SIZE];
        let mut heap = BinaryHeap::new();
        let mut came_from = vec![vec![None; GRID_SIZE]; GRID_SIZE];

        dist[start.0][start.1] = 0;
        heap.push(PathState {
            cost: 0,
            position: start,
        });

        let directions: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        while let Some(PathState { cost, position }) = heap.pop() {
            if position == goal {
                break;
            }
            if cost > dist[position.0][position.1] {
                continue;
            }

            for (dx, dy) in directions.iter() {
                let nx = position.0 as isize + dx;
                let ny = position.1 as isize + dy;

                if nx >= 0 && nx < GRID_SIZE as isize && ny >= 0 && ny < GRID_SIZE as isize {
                    let (ux, uy) = (nx as usize, ny as usize);
                    let terrain_cost = if self.grid[ux][uy] { 1 } else { 50 };
                    let next_cost = cost + terrain_cost;

                    if next_cost < dist[ux][uy] {
                        heap.push(PathState {
                            cost: next_cost,
                            position: (ux, uy),
                        });
                        dist[ux][uy] = next_cost;
                        came_from[ux][uy] = Some(position);
                    }
                }
            }
        }

        let mut path = Vec::new();
        let mut curr = goal;
        if came_from[curr.0][curr.1].is_none() {
            return path;
        }

        while curr != start {
            path.push(curr);
            curr = came_from[curr.0][curr.1].unwrap();
        }
        path
    }
}
