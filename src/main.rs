mod app;
mod network;
mod pager_simulation;
mod player;
mod simulation_environment;

fn main() -> eframe::Result<()>
{
    eframe::run_native(
        "Pager Message Simulator",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(app::PagerSimulationApp::new(cc))))
    )
}
