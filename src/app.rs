use eframe::egui;

use crate::pager_simulation::PagerSimulation;
use crate::simulation_environment::GRID_SIZE;

const DEFAULT_NUMBER_OF_PEOPLE: usize = 170;
const DEFAULT_TRANSMISSION_RANGE_UNITS: f32 = 3.0;

pub struct PagerSimulationApp
{
    sim:                      PagerSimulation,
    transmission_range_units: f32,
    number_of_people:         usize
}

impl PagerSimulationApp
{
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self
    {
        Self {
            sim:                      PagerSimulation::new(DEFAULT_NUMBER_OF_PEOPLE),
            transmission_range_units: DEFAULT_TRANSMISSION_RANGE_UNITS,
            number_of_people:         DEFAULT_NUMBER_OF_PEOPLE
        }
    }
}

impl eframe::App for PagerSimulationApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        self.sim.step(self.transmission_range_units);

        egui::SidePanel::left("controls")
            .exact_width(284.0)
            .show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.transmission_range_units, 1.1..=8.0)
                        .text("Transmission Range")
                );

                if ui
                    .add(egui::Slider::new(&mut self.number_of_people, 50..=400).text("Crowd Size"))
                    .changed()
                {
                    self.sim = PagerSimulation::new(self.number_of_people);
                }

                if ui.button("send pager message").clicked()
                {
                    self.sim.send_pager_message();
                }
                if ui.button("Reset").clicked()
                {
                    self.sim = PagerSimulation::new(self.number_of_people);
                }

                ui.label("% received pager message");

                let (graph_res, painter) =
                    ui.allocate_painter(egui::vec2(230.0, 150.0), egui::Sense::hover());
                painter.rect_filled(graph_res.rect, 2.0, egui::Color32::from_gray(30));

                // thanks egui for not being able to render an empty set!
                if self.sim.successful_transmission_percentages_per_tick.len() > 1
                {
                    let pts: Vec<egui::Pos2> = self
                        .sim
                        .successful_transmission_percentages_per_tick
                        .iter()
                        .enumerate()
                        .map(|(i, &sat)| {
                            egui::pos2(
                                graph_res.rect.left()
                                    + (i as f32
                                        * (graph_res.rect.width()
                                            / self
                                                .sim
                                                .successful_transmission_percentages_per_tick
                                                .len()
                                                as f32)),
                                graph_res.rect.bottom() - (sat * graph_res.rect.height())
                            )
                        })
                        .collect();
                    painter.add(egui::Shape::line(
                        pts,
                        egui::Stroke::new(2.0_f32, egui::Color32::GREEN)
                    ));
                    ui.label(format!(
                        "{:.1}%",
                        self.sim
                            .successful_transmission_percentages_per_tick
                            .last()
                            .unwrap()
                            * 100.0
                    ));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (res, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let cell_size = (res.rect.width().min(res.rect.height()) / GRID_SIZE as f32) * 0.95;
            let offset = res.rect.min
                + egui::vec2(
                    (res.rect.width() - (cell_size * GRID_SIZE as f32)) / 2.0,
                    (res.rect.height() - (cell_size * GRID_SIZE as f32)) / 2.0
                );

            for x in 0..GRID_SIZE
            {
                for y in 0..GRID_SIZE
                {
                    if !self.sim.env.is_street_grid[x][y]
                    {
                        let min = offset + egui::vec2(x as f32 * cell_size, y as f32 * cell_size);
                        painter.rect_filled(
                            egui::Rect::from_min_max(min, min + egui::vec2(cell_size, cell_size)),
                            0.0,
                            egui::Color32::from_gray(40)
                        );
                    }
                }
            }

            for m in &self.sim.recently_transmitted_messages_positions
            {
                let o = offset
                    + egui::vec2(
                        m.origin.x * cell_size + cell_size / 2.0,
                        m.origin.y * cell_size + cell_size / 2.0
                    );
                let d = offset
                    + egui::vec2(
                        m.destination.x * cell_size + cell_size / 2.0,
                        m.destination.y * cell_size + cell_size / 2.0
                    );
                let color = egui::Color32::from_rgba_unmultiplied(
                    0,
                    255,
                    255,
                    (255.0 * (m.frames_remaining as f32 / 60.0)) as u8
                );
                painter.line_segment([o, d], egui::Stroke::new(1.5_f32, color));
            }

            for p in &self.sim.people
            {
                let pos = offset
                    + egui::vec2(
                        p.position.x * cell_size + cell_size / 2.0,
                        p.position.y * cell_size + cell_size / 2.0
                    );
                let color = if self.sim.active_message.is_some()
                    && p.seen_messages.contains(&self.sim.active_message.unwrap())
                {
                    egui::Color32::GREEN
                }
                else
                {
                    egui::Color32::GRAY
                };
                painter.circle_filled(pos, cell_size * 0.3, color);
            }
        });

        ctx.request_repaint();
    }
}
