use uuid::Uuid;

// Updated import paths
use crate::network::{VisualTransmissionEvent, resolve_transmissions};
use crate::player::Player;
use crate::simulation_environment::Environment;

pub struct PagerSimulation
{
    pub env:                Environment,
    pub players:            Vec<Player>,
    pub visual_events:      Vec<VisualTransmissionEvent>,
    pub active_message:     Option<Uuid>,
    pub saturation_history: Vec<f32>
}

impl PagerSimulation
{
    pub fn new(agent_count: usize) -> Self
    {
        let env = Environment::new();
        let mut players = Vec::new();
        for _ in 0..agent_count
        {
            players.push(Player::new(&env));
        }

        Self {
            env,
            players,
            visual_events: Vec::new(),
            active_message: None,
            saturation_history: Vec::new()
        }
    }

    pub fn inject_broadcast(&mut self)
    {
        let new_uuid = Uuid::new_v4();
        self.active_message = Some(new_uuid);
        self.saturation_history.clear();

        if let Some(first_agent) = self.players.first_mut()
        {
            first_agent.seen_messages.insert(new_uuid);
        }
    }

    pub fn clear_network(&mut self)
    {
        self.active_message = None;
        self.saturation_history.clear();
        for p in &mut self.players
        {
            p.seen_messages.clear();
        }
    }

    pub fn step(&mut self, transmission_range: f32)
    {
        for p in &mut self.players
        {
            p.movement_update(&self.env);
        }

        if let Some(uuid) = self.active_message
        {
            let new_events = resolve_transmissions(&mut self.players, uuid, transmission_range);
            self.visual_events.extend(new_events);

            let infected = self
                .players
                .iter()
                .filter(|p| p.seen_messages.contains(&uuid))
                .count();
            self.saturation_history
                .push(infected as f32 / self.players.len() as f32);
        }

        self.visual_events.retain_mut(|e| {
            e.frames_remaining = e.frames_remaining.saturating_sub(1);
            e.frames_remaining > 0
        });
    }
}
