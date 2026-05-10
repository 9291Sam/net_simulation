use std::collections::HashSet;

use glam::Vec2;
use rand::Rng;
use uuid::Uuid;

use crate::simulation_environment::Environment;

pub const AGENT_SPEED_UNITS_PER_TICK: f32 = 0.1;

#[derive(Clone)]
pub struct Player
{
    pub position:             Vec2,
    pub active_path:          Vec<(usize, usize)>,
    pub wait_ticks_remaining: u32,
    pub seen_messages:        HashSet<Uuid>
}

impl Player
{
    pub fn new(env: &Environment) -> Self
    {
        let mut rng = rand::thread_rng();
        let spawn_loc = env.get_random_building_cell();

        Self {
            position:             glam::vec2(spawn_loc.0 as f32, spawn_loc.1 as f32),
            active_path:          Vec::new(),
            wait_ticks_remaining: rng.gen_range(0..3600),
            seen_messages:        HashSet::new()
        }
    }

    pub fn movement_update(&mut self, env: &Environment)
    {
        if self.wait_ticks_remaining > 0
        {
            self.wait_ticks_remaining -= 1;
            return;
        }

        let mut rng = rand::thread_rng();

        // 1. If we have no path, generate one and start moving immediately
        if self.active_path.is_empty()
        {
            let target_cell = if rng.gen_range(0.0..1.0) < 0.7
            {
                env.hotspots[rng.gen_range(0..4)]
            }
            else
            {
                env.get_random_building_cell()
            };

            let current_grid_x = self.position.x.round() as usize;
            let current_grid_y = self.position.y.round() as usize;

            self.active_path = env.calculate_path((current_grid_x, current_grid_y), target_cell);

            // Fallback: if the path failed to generate, wait 1 second and try again next
            // tick
            if self.active_path.is_empty()
            {
                self.wait_ticks_remaining = 60;
                return;
            }
        }

        // 2. We have a path, process movement
        let next_node = *self.active_path.last().unwrap();
        let target_pos = glam::vec2(next_node.0 as f32, next_node.1 as f32);

        let to_target = target_pos - self.position;
        let distance = to_target.length();

        if distance <= AGENT_SPEED_UNITS_PER_TICK
        {
            // Reached the grid cell
            self.position = target_pos;
            self.active_path.pop();

            // FIX: Assign the long indoor wait timer ONLY upon arriving at the final
            // destination
            if self.active_path.is_empty()
            {
                self.wait_ticks_remaining = rng.gen_range(600..3600);
            }
        }
        else
        {
            // Transit smoothly towards the next cell
            let direction = to_target.normalize();
            self.position += direction * AGENT_SPEED_UNITS_PER_TICK;
        }
    }
}
