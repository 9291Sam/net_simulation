use std::cmp::Ordering;
use std::collections::BinaryHeap;

use rand::Rng;

pub const GRID_SIZE: usize = 65;
pub const NUMBER_OF_HOTSPOTS: usize = 20;

#[derive(Copy, Clone, Eq, PartialEq)]
struct PathWithCost
{
    cost:     usize,
    position: (usize, usize)
}

impl Ord for PathWithCost
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for PathWithCost
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}

pub struct Environment
{
    pub is_street_grid: [[bool; GRID_SIZE]; GRID_SIZE],
    pub hotspots:       Vec<(usize, usize)>
}

impl Environment
{
    pub fn new() -> Self
    {
        let mut env = Self {
            is_street_grid: [[false; GRID_SIZE]; GRID_SIZE],
            hotspots:       vec![]
        };

        // TODO: add alternative city shapes?
        for x in 0..GRID_SIZE
        {
            for y in 0..GRID_SIZE
            {
                if x % 5 == 0 || y % 5 == 0
                {
                    env.is_street_grid[x][y] = true;
                }
            }
        }

        for _ in 0..NUMBER_OF_HOTSPOTS
        {
            env.hotspots.push(env.get_random_building_cell());
        }

        env
    }

    pub fn get_random_building_cell(&self) -> (usize, usize)
    {
        // TODO: if this is a problem put the non street grids in a list and do a choose
        // from that
        loop
        {
            let x = rand::thread_rng().gen_range(0..GRID_SIZE);
            let y = rand::thread_rng().gen_range(0..GRID_SIZE);

            if !self.is_street_grid[x][y]
            {
                return (x, y);
            }
        }
    }

    pub fn calculate_path(&self, start: (usize, usize), goal: (usize, usize))
    -> Vec<(usize, usize)>
    {
        if start == goal
        {
            return vec![];
        }

        let mut dist = vec![vec![usize::MAX; GRID_SIZE]; GRID_SIZE];
        let mut heap = BinaryHeap::new();
        let mut came_from = vec![vec![None; GRID_SIZE]; GRID_SIZE];

        dist[start.0][start.1] = 0;

        heap.push(PathWithCost {
            cost:     0,
            position: start
        });

        // TODO: enum for this?
        let directions: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        while let Some(PathWithCost {
            cost,
            position
        }) = heap.pop()
        {
            if position == goal
            {
                break;
            }

            if cost > dist[position.0][position.1]
            {
                continue;
            }

            for (dx, dy) in directions.iter()
            {
                let nx = position.0 as isize + dx;
                let ny = position.1 as isize + dy;

                if nx >= 0 && nx < GRID_SIZE as isize && ny >= 0 && ny < GRID_SIZE as isize
                {
                    let (ux, uy) = (nx as usize, ny as usize);

                    let terrain_cost = if self.is_street_grid[ux][uy]
                    {
                        1
                    }
                    else
                    {
                        50
                    };

                    let next_cost = cost + terrain_cost;

                    if next_cost < dist[ux][uy]
                    {
                        heap.push(PathWithCost {
                            cost:     next_cost,
                            position: (ux, uy)
                        });

                        dist[ux][uy] = next_cost;

                        came_from[ux][uy] = Some(position);
                    }
                }
            }
        }

        let mut path = vec![];
        let mut curr = goal;

        if came_from[curr.0][curr.1].is_none()
        {
            return path;
        }

        while curr != start
        {
            path.push(curr);
            curr = came_from[curr.0][curr.1].unwrap();
        }

        path
    }
}
