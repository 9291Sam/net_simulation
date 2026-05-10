use std::collections::HashSet;

use glam::Vec2;
use rand::Rng;
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::simulation_environment::Environment;

pub const PERSON_MOVE_SPEED_PER_TICK: f32 = 0.1;
pub const CYCLE_TIME_LOWER_BOUND_TICKS: u32 = 600;
pub const CYCLE_TIME_UPPER_BOUND_TICKS: u32 = 3600;
pub const HOTSPOT_TRAVEL_PROBABILITY: f32 = 0.8;

#[derive(Clone)]
pub struct Person
{
    pub position:             Vec2,
    pub active_path:          Vec<(usize, usize)>,
    pub wait_ticks_remaining: u32,
    pub seen_messages:        HashSet<Uuid> // TODO: multiple messages and/or acks?
}

impl Person
{
    pub fn new(env: &Environment) -> Self
    {
        let spawn_loc = env.get_random_building_cell();

        Self {
            position:             glam::vec2(spawn_loc.0 as f32, spawn_loc.1 as f32),
            active_path:          vec![],
            wait_ticks_remaining: rand::thread_rng().gen_range(0..CYCLE_TIME_UPPER_BOUND_TICKS),
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

        if self.active_path.is_empty()
        {
            let target_cell = if rand::thread_rng().gen_range(0.0..1.0) < HOTSPOT_TRAVEL_PROBABILITY
            {
                *env.hotspots.choose(&mut rand::thread_rng()).unwrap()
            }
            else
            {
                env.get_random_building_cell()
            };

            self.active_path = env.calculate_path(
                (
                    self.position.x.round() as usize,
                    self.position.y.round() as usize
                ),
                target_cell
            );

            if self.active_path.is_empty()
            {
                // dumb stupid dumb bug, you can route to yourself, this took too long to track
                // down
                return;
            }
        }

        // normal movement
        if let Some(&next_node) = self.active_path.last()
        {
            let target_position = glam::vec2(next_node.0 as f32, next_node.1 as f32);
            let vector_to_target = target_position - self.position;
            let distance_to_target = vector_to_target.length();

            // TODO: I hate this but it works good enough and you can't notice the fact that
            // it jerks a little at points.
            // TODO: replace with a continuous thing to avoid this
            if distance_to_target <= PERSON_MOVE_SPEED_PER_TICK
            {
                self.position = target_position;
                self.active_path.pop();

                if self.active_path.is_empty()
                {
                    self.wait_ticks_remaining = rand::thread_rng()
                        .gen_range(CYCLE_TIME_LOWER_BOUND_TICKS..CYCLE_TIME_UPPER_BOUND_TICKS);
                }
            }
            else
            {
                self.position += vector_to_target.normalize() * PERSON_MOVE_SPEED_PER_TICK;
            }
        }
    }
}
