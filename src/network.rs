use glam::Vec2;
use rand::Rng;
use uuid::Uuid;

use crate::player::Player;

pub const SIGNAL_DECAY_LAMBDA: f32 = 0.5;

pub struct VisualTransmissionEvent
{
    pub origin:           Vec2,
    pub destination:      Vec2,
    pub frames_remaining: u8
}

pub fn resolve_transmissions(
    players: &mut [Player],
    active_uuid: Uuid,
    range_units: f32
) -> Vec<VisualTransmissionEvent>
{
    let mut rng = rand::thread_rng();
    let mut new_infections = Vec::new();

    for i in 0..players.len()
    {
        if !players[i].seen_messages.contains(&active_uuid)
        {
            continue;
        }

        for j in 0..players.len()
        {
            if i == j || players[j].seen_messages.contains(&active_uuid)
            {
                continue;
            }

            let distance = players[i].position.distance(players[j].position);

            if distance <= range_units
            {
                let probability = (-SIGNAL_DECAY_LAMBDA * distance).exp();
                if rng.gen_range(0.0..1.0) < probability
                {
                    new_infections.push((j, players[i].position, players[j].position));
                }
            }
        }
    }

    let mut events = Vec::new();
    for (target_index, origin, destination) in new_infections
    {
        players[target_index].seen_messages.insert(active_uuid);
        events.push(VisualTransmissionEvent {
            origin,
            destination,
            frames_remaining: 60
        });
    }

    events
}
