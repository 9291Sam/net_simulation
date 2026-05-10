use eframe::egui;
// Updated import paths
use crate::pager_simulation::PagerSimulation;
use crate::simulation_environment::GRID_SIZE;

pub struct DtnUrbanSimApp {
    sim: PagerSimulation,
    transmission_range_units: f32,
    target_agent_count: usize,
}

impl DtnUrbanSimApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            sim: PagerSimulation::new(200),
            transmission_range_units: 3.0,
            target_agent_count: 200,
        }
    }
}

impl eframe::App for DtnUrbanSimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sim.step(self.transmission_range_units);

        egui::SidePanel::left("controls")
            .exact_width(250.0)
            .show(ctx, |ui| {
                ui.heading("DTN Simulation");
                ui.separator();

                ui.add(
                    egui::Slider::new(&mut self.transmission_range_units, 1.0..=8.0)
                        .text("Tx Range"),
                );

                let mut requested_count = self.target_agent_count;
                if ui
                    .add(egui::Slider::new(&mut requested_count, 50..=400).text("Crowd Size"))
                    .changed()
                {
                    self.target_agent_count = requested_count;
                    self.sim.reset_players(self.target_agent_count);
                }

                ui.add_space(20.0);
                if ui.button("Inject Broadcast").clicked() {
                    self.sim.inject_broadcast();
                }
                if ui.button("Clear Network").clicked() {
                    self.sim.clear_network();
                }

                ui.add_space(20.0);
                ui.label("Infection Timeline:");

                let (graph_res, painter) =
                    ui.allocate_painter(egui::vec2(230.0, 150.0), egui::Sense::hover());
                painter.rect_filled(graph_res.rect, 2.0, egui::Color32::from_gray(30));

                if self.sim.saturation_history.len() > 1 {
                    let pts: Vec<egui::Pos2> = self
                        .sim
                        .saturation_history
                        .iter()
                        .enumerate()
                        .map(|(i, &sat)| {
                            egui::pos2(
                                graph_res.rect.left()
                                    + (i as f32
                                        * (graph_res.rect.width()
                                            / self.sim.saturation_history.len() as f32)),
                                graph_res.rect.bottom() - (sat * graph_res.rect.height()),
                            )
                        })
                        .collect();
                    painter.add(egui::Shape::line(
                        pts,
                        egui::Stroke::new(2.0, egui::Color32::GREEN),
                    ));
                    ui.label(format!(
                        "Saturation: {:.1}%",
                        self.sim.saturation_history.last().unwrap() * 100.0
                    ));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (res, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let cell_size = (res.rect.width().min(res.rect.height()) / GRID_SIZE as f32) * 0.95;
            let offset = res.rect.min
                + egui::vec2(
                    (res.rect.width() - (cell_size * GRID_SIZE as f32)) / 2.0,
                    (res.rect.height() - (cell_size * GRID_SIZE as f32)) / 2.0,
                );

            for x in 0..GRID_SIZE {
                for y in 0..GRID_SIZE {
                    if !self.sim.env.grid[x][y] {
                        let min = offset + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                        painter.rect_filled(
                            egui::Rect::from_min_max(min, min + egui::vec2(cell_size, cell_size)),
                            0.0,
                            egui::Color32::from_gray(40),
                        );
                    }
                }
            }

            for event in &self.sim.visual_events {
                let o = offset
                    + egui::vec2(
                        event.origin.x * cell_size + cell_size / 2.0,
                        event.origin.y * cell_size + cell_size / 2.0,
                    );
                let d = offset
                    + egui::vec2(
                        event.destination.x * cell_size + cell_size / 2.0,
                        event.destination.y * cell_size + cell_size / 2.0,
                    );
                let color = egui::Color32::from_rgba_unmultiplied(
                    0,
                    255,
                    255,
                    (255.0 * (event.frames_remaining as f32 / 60.0)) as u8,
                );
                painter.line_segment([o, d], egui::Stroke::new(1.5, color));
            }

            for p in &self.sim.players {
                let pos = offset
                    + egui::vec2(
                        p.position.x * cell_size + cell_size / 2.0,
                        p.position.y * cell_size + cell_size / 2.0,
                    );
                let color = if self.sim.active_message.is_some()
                    && p.seen_messages.contains(&self.sim.active_message.unwrap())
                {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                };
                painter.circle_filled(pos, cell_size * 0.3, color);
            }
        });

        ctx.request_repaint();
    }
}
