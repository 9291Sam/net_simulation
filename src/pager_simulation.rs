use rand::seq::SliceRandom;
use uuid::Uuid;

// Updated import paths
use crate::network::{MessageSentBetweenPositions, resolve_transmissions};
use crate::person::Person;
use crate::simulation_environment::Environment;

pub struct PagerSimulation
{
    pub env: Environment,
    pub people: Vec<Person>,
    pub recently_transmitted_messages_positions: Vec<MessageSentBetweenPositions>,
    pub active_message: Option<Uuid>,
    pub successful_transmission_percentages_per_tick: Vec<f32>
}

impl PagerSimulation
{
    pub fn new(people_count: usize) -> Self
    {
        let env = Environment::new();

        let people = (0..people_count).map(|_| Person::new(&env)).collect();

        Self {
            env,
            people,
            recently_transmitted_messages_positions: vec![],
            active_message: None,
            successful_transmission_percentages_per_tick: vec![]
        }
    }

    pub fn send_pager_message(&mut self)
    {
        let new_message_uuid = Uuid::new_v4();

        self.active_message = Some(new_message_uuid);

        // TODO: it's a code smell that this is here, but idgaf rn!
        self.successful_transmission_percentages_per_tick.clear();

        if let Some(p) = self.people.choose_mut(&mut rand::thread_rng())
        {
            p.seen_messages.insert(new_message_uuid);
        }
    }

    pub fn step(&mut self, transmission_range: f32)
    {
        for p in &mut self.people
        {
            p.movement_update(&self.env);
        }

        if let Some(uuid) = self.active_message
        {
            let places_where_messages_were_transmitted =
                resolve_transmissions(&mut self.people, uuid, transmission_range);

            self.recently_transmitted_messages_positions
                .extend(places_where_messages_were_transmitted);

            let transmission = self
                .people
                .iter()
                .filter(|p| p.seen_messages.contains(&uuid))
                .count();

            self.successful_transmission_percentages_per_tick
                .push(transmission as f32 / self.people.len() as f32);
        }

        self.recently_transmitted_messages_positions
            .retain_mut(|e| {
                e.frames_remaining = e.frames_remaining.saturating_sub(1);
                e.frames_remaining > 0
            });
    }
}
