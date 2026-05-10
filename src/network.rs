use glam::Vec2;
use rand::Rng;
use uuid::Uuid;

use crate::person::Person;

pub const SIGNAL_DECAY_LAMBDA: f32 = 0.5;
pub const TIME_THAT_BLUE_LINE_EXISTS_FOR: u8 = 60;

pub struct MessageSentBetweenPositions
{
    pub origin:           Vec2,
    pub destination:      Vec2,
    pub frames_remaining: u8
}

pub fn resolve_transmissions(
    people: &mut [Person],
    active_uuid: Uuid,
    range_units: f32
) -> Vec<MessageSentBetweenPositions>
{
    let mut rng = rand::thread_rng();
    let mut new_transmissions = vec![];

    for i in 0..people.len()
    {
        if !people[i].seen_messages.contains(&active_uuid)
        {
            continue;
        }

        for j in 0..people.len()
        {
            if i == j || people[j].seen_messages.contains(&active_uuid)
            {
                continue;
            }

            let distance = people[i].position.distance(people[j].position);

            if distance <= range_units
            {
                let probability = (-SIGNAL_DECAY_LAMBDA * distance).exp();
                if rng.gen_range(0.0..1.0) < probability
                {
                    new_transmissions.push((j, people[i].position, people[j].position));
                }
            }
        }
    }

    let mut events = vec![];

    for (target_index, origin, destination) in new_transmissions
    {
        people[target_index].seen_messages.insert(active_uuid);

        events.push(MessageSentBetweenPositions {
            origin,
            destination,
            frames_remaining: TIME_THAT_BLUE_LINE_EXISTS_FOR
        });
    }

    events
}
