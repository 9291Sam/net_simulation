use eframe::egui;
use rand::rng;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
struct NetworkPayload {
    uuid: Uuid,
    is_acknowledge_of_other: Option<Uuid>,
    from: hecs::Entity,
    to: hecs::Entity,
    time_to_live: u32,
}

enum PersonState {
    Sleeping,
    Home,
    Work,
    Socialize,
    RandomMove,
    Errand,
    InTransit,
}

const BUILDING_GRID_SIZE: u8 = 12;

#[derive(Clone, Copy, Debug, Default, Hash)]
struct BuildingLocation(u8);

impl BuildingLocation {
    fn get_grid(self) -> glam::U8Vec2 {
        glam::u8vec2(self.0 % BUILDING_GRID_SIZE, self.0 / 12)
    }
}

enum MovingState {
    Stationary(BuildingLocation),
    Moving {
        from: BuildingLocation,
        to: BuildingLocation,
        ticks_since_start: u32,
    },
}

struct Person {
    age: u8,
    state: PersonState,
    acquaintances: Vec<hecs::Entity>,
    home_location: BuildingLocation,
    work_location: BuildingLocation,
    socialization_locations: Vec<BuildingLocation>,
    global_time_offset_ticks: u32,
    moving_state: Option<MovingState>,
    recent_messages: HashMap<Uuid, (u32, NetworkPayload)>,
}

impl Person {
    fn get_continious_position(&self) -> glam::Vec2 {
        match &self.moving_state {
            Some(MovingState::Stationary(location)) => {
                let grid_coordinate: glam::U8Vec2 = location.get_grid();
                glam::vec2(grid_coordinate.x as f32, grid_coordinate.y as f32)
            }
            Some(MovingState::Moving {
                from,
                to,
                ticks_since_start,
            }) => {
                let starting_grid_coordinate: glam::Vec2 = from.get_grid().as_vec2();
                let destination_grid_coordinate: glam::Vec2 = to.get_grid().as_vec2();

                let total_distance_units: f32 =
                    starting_grid_coordinate.distance(destination_grid_coordinate);
                let required_transit_ticks: f32 =
                    (total_distance_units * TICKS_PER_UNIT_DISTANCE as f32).ceil();

                if required_transit_ticks == 0.0 {
                    return starting_grid_coordinate;
                }

                let movement_progress_ratio: f32 =
                    (*ticks_since_start as f32) / required_transit_ticks;
                let clamped_movement_progress_ratio: f32 = movement_progress_ratio.clamp(0.0, 1.0);

                starting_grid_coordinate
                    .lerp(destination_grid_coordinate, clamped_movement_progress_ratio)
            }
            None => {
                let grid_coordinate: glam::U8Vec2 = self.home_location.get_grid();
                glam::vec2(grid_coordinate.x as f32, grid_coordinate.y as f32)
            }
        }
    }
}

const TICKS_PER_DAY: u32 = 17280;
const TICKS_PER_UNIT_DISTANCE: u32 = 36;

fn distance_between_building_locations(l: BuildingLocation, r: BuildingLocation) -> f32 {
    let grid_l: glam::U8Vec2 = l.get_grid();
    let grid_r: glam::U8Vec2 = r.get_grid();

    let dx: f32 = grid_l.x as f32 - grid_r.x as f32;
    let dy: f32 = grid_l.y as f32 - grid_r.y as f32;

    (dx * dx + dy * dy).sqrt()
}

fn update_person_schedule(current_global_tick: u32, person: &mut Person) {
    let local_tick_index: u32 =
        (current_global_tick + person.global_time_offset_ticks) % TICKS_PER_DAY;
    let local_day_index: u32 =
        (current_global_tick + person.global_time_offset_ticks) / TICKS_PER_DAY;

    let (desired_state, target_location): (PersonState, BuildingLocation) = match local_tick_index {
        0..=5040 => (PersonState::Sleeping, person.home_location),
        5041..=5760 => (PersonState::Home, person.home_location),
        5761..=12240 => (PersonState::Work, person.work_location),
        12241..=13680 => {
            if person.socialization_locations.is_empty() {
                (PersonState::Home, person.home_location)
            } else {
                (
                    PersonState::Socialize,
                    person.socialization_locations
                        [(local_day_index % person.socialization_locations.len() as u32) as usize],
                )
            }
        }
        13681..=16560 => (PersonState::Home, person.home_location),
        _ => (PersonState::Sleeping, person.home_location),
    };

    if let Some(current_movement) = person.moving_state.take() {
        match current_movement {
            MovingState::Stationary(current_loc) => {
                if current_loc.0 != target_location.0 {
                    person.moving_state = Some(MovingState::Moving {
                        from: current_loc,
                        to: target_location,
                        ticks_since_start: 0,
                    });
                    person.state = PersonState::InTransit;
                } else {
                    person.moving_state = Some(MovingState::Stationary(current_loc));
                    person.state = desired_state;
                }
            }
            MovingState::Moving {
                from,
                to,
                ticks_since_start,
            } => {
                let next_tick_count: u32 = ticks_since_start + 1;
                let distance_units: f32 = distance_between_building_locations(from, to);
                let required_transit_ticks: u32 =
                    (distance_units * TICKS_PER_UNIT_DISTANCE as f32).ceil() as u32;

                if next_tick_count >= required_transit_ticks {
                    person.moving_state = Some(MovingState::Stationary(to));

                    if to.0 == target_location.0 {
                        person.state = desired_state;
                    } else {
                        person.state = PersonState::InTransit;
                    }
                } else {
                    person.moving_state = Some(MovingState::Moving {
                        from,
                        to,
                        ticks_since_start: next_tick_count,
                    });
                    person.state = PersonState::InTransit;
                }
            }
        }
    } else {
        person.moving_state = Some(MovingState::Stationary(person.home_location));
        person.state = PersonState::Home;
    }
}

const MESSAGE_TIME_TO_LIVE_TICKS: u32 = 36;

fn process_network_culling(current_global_tick: u32, person: &mut Person) {
    let mut acknowledged_message_identifiers: Vec<Uuid> = Vec::new();

    for (_, payload) in person.recent_messages.values() {
        if let Some(ack_uuid) = payload.is_acknowledge_of_other {
            acknowledged_message_identifiers.push(ack_uuid);
        }
    }

    person
        .recent_messages
        .retain(|message_uuid, (tick_appeared, _)| {
            let message_age_ticks: u32 = current_global_tick.saturating_sub(*tick_appeared);
            let has_expired: bool = message_age_ticks >= MESSAGE_TIME_TO_LIVE_TICKS;
            let has_been_acknowledged: bool =
                acknowledged_message_identifiers.contains(message_uuid);

            !(has_expired || has_been_acknowledged)
        });
}

struct AgentNetworkSnapshot {
    entity_handle: hecs::Entity,
    continuous_spatial_coordinate: glam::Vec2,
    active_message_payloads: Vec<NetworkPayload>,
    acquaintance_entity_handles: Vec<hecs::Entity>,
}

const MAXIMUM_TRANSMISSION_RANGE_UNITS: f32 = 1.5;
const SIGNAL_EXPONENTIAL_DECAY_CONSTANT: f32 = 0.5;

fn execute_network_propagation_phase(current_global_tick: u32, ecs_world: &mut hecs::World) {
    let mut network_snapshots: Vec<AgentNetworkSnapshot> = Vec::new();

    for (entity_handle, person_component) in ecs_world.query_mut::<&mut Person>() {
        let extracted_payloads: Vec<NetworkPayload> = person_component
            .recent_messages
            .values()
            .map(|(_, payload)| payload.clone())
            .collect();

        network_snapshots.push(AgentNetworkSnapshot {
            entity_handle,
            continuous_spatial_coordinate: person_component.get_continious_position(),
            active_message_payloads: extracted_payloads,
            acquaintance_entity_handles: person_component.acquaintances.clone(),
        });
    }

    let mut pending_message_deliveries: Vec<(hecs::Entity, NetworkPayload)> = Vec::new();

    for transmitting_agent in network_snapshots.iter() {
        if transmitting_agent.active_message_payloads.is_empty() {
            continue;
        }

        for receiving_agent in network_snapshots.iter() {
            if transmitting_agent.entity_handle == receiving_agent.entity_handle {
                continue;
            }

            let spatial_distance: f32 = transmitting_agent
                .continuous_spatial_coordinate
                .distance(receiving_agent.continuous_spatial_coordinate);

            if spatial_distance > MAXIMUM_TRANSMISSION_RANGE_UNITS {
                continue;
            }

            let mut transmission_success_probability: f32 =
                (-SIGNAL_EXPONENTIAL_DECAY_CONSTANT * spatial_distance).exp();

            if receiving_agent
                .acquaintance_entity_handles
                .contains(&transmitting_agent.entity_handle)
            {
                transmission_success_probability *= 1.5;
            }

            if rand::random::<f32>() < transmission_success_probability {
                for payload in transmitting_agent.active_message_payloads.iter() {
                    pending_message_deliveries
                        .push((receiving_agent.entity_handle, payload.clone()));
                }
            }
        }
    }

    for (target_entity_handle, incoming_payload) in pending_message_deliveries {
        if let Ok(person_component) = ecs_world.query_one_mut::<&mut Person>(target_entity_handle) {
            if let Some(acknowledged_uuid) = incoming_payload.is_acknowledge_of_other {
                person_component.recent_messages.remove(&acknowledged_uuid);
            }

            person_component
                .recent_messages
                .entry(incoming_payload.uuid)
                .or_insert((current_global_tick, incoming_payload));
        }
    }
}

fn generate_random_building_location() -> BuildingLocation {
    BuildingLocation(rand::random::<u8>() % (BUILDING_GRID_SIZE * BUILDING_GRID_SIZE))
}

fn initialize_simulation_world(total_agent_count: usize) -> hecs::World {
    let mut ecs_world = hecs::World::new();
    let mut spawned_entity_handles: Vec<hecs::Entity> = Vec::with_capacity(total_agent_count);

    for _ in 0..total_agent_count {
        let home_loc: BuildingLocation = generate_random_building_location();
        let work_loc: BuildingLocation = generate_random_building_location();

        let mut socialization_locations: Vec<BuildingLocation> = Vec::new();
        for _ in 0..(rand::random::<u64>() % 4) {
            socialization_locations.push(generate_random_building_location());
        }

        let entity_handle: hecs::Entity = ecs_world.spawn((Person {
            age: 20 + (rand::random::<u8>() % 40),
            state: PersonState::Home,
            acquaintances: Vec::new(),
            home_location: home_loc,
            work_location: work_loc,
            socialization_locations,
            global_time_offset_ticks: rand::random::<u32>() % TICKS_PER_DAY,
            moving_state: None,
            recent_messages: HashMap::new(),
        },));
        spawned_entity_handles.push(entity_handle);
    }

    for entity_handle in &spawned_entity_handles {
        let mut acquaintances: Vec<hecs::Entity> = Vec::new();
        let connection_count: u64 = 2 + (rand::random::<u64>() % 8);
        for _ in 0..connection_count {
            let target_handle: hecs::Entity =
                spawned_entity_handles[(rand::random::<u64>() % total_agent_count as u64) as usize];
            if target_handle != *entity_handle && !acquaintances.contains(&target_handle) {
                acquaintances.push(target_handle);
            }
        }

        if let Ok(mut person_component) = ecs_world.query_one_mut::<&mut Person>(*entity_handle) {
            person_component.acquaintances = acquaintances;
        }
    }

    ecs_world
}

struct DtnSimulationApp {
    ecs_world: hecs::World,
    current_global_tick: u32,
    simulation_ticks_per_frame: u32,
}

impl DtnSimulationApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            ecs_world: initialize_simulation_world(1000),
            current_global_tick: 0,
            simulation_ticks_per_frame: 1,
        }
    }
}

impl eframe::App for DtnSimulationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for _ in 0..self.simulation_ticks_per_frame {
            for (_, person_component) in self.ecs_world.query_mut::<&mut Person>() {
                update_person_schedule(self.current_global_tick, person_component);
                process_network_culling(self.current_global_tick, person_component);
            }
            execute_network_propagation_phase(self.current_global_tick, &mut self.ecs_world);
            self.current_global_tick += 1;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("inject payload to random agent").clicked() {
                let random_agent_index: usize = (rand::random::<u64>() % 1000) as usize;
                let target_entity: hecs::Entity = self
                    .ecs_world
                    .iter()
                    .nth(random_agent_index)
                    .unwrap()
                    .entity();
                let payload_uuid: Uuid = Uuid::new_v4();

                if let Ok(mut person_component) =
                    self.ecs_world.query_one_mut::<&mut Person>(target_entity)
                {
                    person_component.recent_messages.insert(
                        payload_uuid,
                        (
                            self.current_global_tick,
                            NetworkPayload {
                                uuid: payload_uuid,
                                is_acknowledge_of_other: None,
                                from: target_entity,
                                to: target_entity,
                                time_to_live: MESSAGE_TIME_TO_LIVE_TICKS,
                            },
                        ),
                    );
                }
            }

            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let render_area_rect: egui::Rect = response.rect;

            let grid_cell_width: f32 = render_area_rect.width() / (BUILDING_GRID_SIZE as f32);
            let grid_cell_height: f32 = render_area_rect.height() / (BUILDING_GRID_SIZE as f32);

            let center_offset_x: f32 = grid_cell_width / 2.0;
            let center_offset_y: f32 = grid_cell_height / 2.0;

            for x in 0..BUILDING_GRID_SIZE {
                for y in 0..BUILDING_GRID_SIZE {
                    let cell_min: egui::Pos2 = render_area_rect.min
                        + egui::vec2(x as f32 * grid_cell_width, y as f32 * grid_cell_height);
                    let cell_max: egui::Pos2 =
                        cell_min + egui::vec2(grid_cell_width, grid_cell_height);
                    painter.rect_stroke(
                        egui::Rect::from_min_max(cell_min, cell_max),
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                    );
                }
            }

            for (_, person_component) in self.ecs_world.query_mut::<&mut Person>() {
                let continuous_position: glam::Vec2 = person_component.get_continious_position();
                let screen_x: f32 = render_area_rect.min.x
                    + (continuous_position.x * grid_cell_width)
                    + center_offset_x;
                let screen_y: f32 = render_area_rect.min.y
                    + (continuous_position.y * grid_cell_height)
                    + center_offset_y;

                let render_color: egui::Color32 = if person_component.recent_messages.is_empty() {
                    egui::Color32::GRAY
                } else {
                    egui::Color32::GREEN
                };

                painter.circle_filled(egui::pos2(screen_x, screen_y), 3.0, render_color);
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "dtn sim",
        native_options,
        Box::new(|cc| Box::new(DtnSimulationApp::new(cc))),
    )
}
