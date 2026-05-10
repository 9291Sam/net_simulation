// Declare the flat module structure
mod app;
mod network;
mod pager_simulation;
mod player;
mod simulation_environment;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "DTN Indoor Clustering Sim",
        native_options,
        Box::new(|cc| Ok(Box::new(app::DtnUrbanSimApp::new(cc)))),
    )
}
