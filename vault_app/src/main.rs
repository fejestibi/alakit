use alakit::AlakitEngine;
use alakit_macro::{alakit_assets, alakit_main};
use std::env;
use std::path::PathBuf;

#[alakit_main]
fn main() {
    // Generate absolute path for the UI directory
    let current_dir = env::current_dir().expect("Failed to get current dir");
    let ui_path = current_dir.join("vault_app/ui");

    let final_ui_path = if ui_path.exists() {
        ui_path.canonicalize().unwrap_or(ui_path)
    } else {
        PathBuf::from("ui")
    };

    println!("Starting Alakit Vault Showcase...");

    AlakitEngine::new("Alakit Vault Showcase")
        .embed_ui(alakit_assets!("ui/"))
        .with_ui_dir(final_ui_path.to_str().unwrap_or("ui"))
        .with_initial_url("index.html")
        .run();
}
