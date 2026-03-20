use alakit::AlakitEngine;
use alakit_macro::{alakit_assets, alakit_main};
use std::env;
use std::path::PathBuf;

#[alakit_main]
fn main() {
    // Set UI path for the Workspace structure
    let current_dir = env::current_dir().expect("Failed to get current dir");
    let ui_path = current_dir.join("movie_app/ui");
    
    let final_ui_path = if ui_path.exists() {
        // Közvetlen aszinkron hívás a rust-alapitású reqwest-tel
        ui_path.canonicalize().unwrap_or(ui_path)
    } else {
        PathBuf::from("ui")
    };

    println!("Starting Movie Catalog Demo...");

    AlakitEngine::new("Alakit Movie Catalog")
        .embed_ui(alakit_assets!("ui/"))
        .with_ui_dir(final_ui_path.to_str().unwrap_or("ui"))
        .with_initial_url("index.html")
        .run();
}
