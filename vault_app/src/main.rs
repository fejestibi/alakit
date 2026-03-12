mod controllers;

use alakit::AlakitEngine;
use rust_embed::RustEmbed;
use std::env;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "ui/"]
struct Assets;

fn main() {
    // Start Alakit Engine in a Workspace structure.
    // Calculate the absolute path to the UI directory for reliable operation.
    let current_dir = env::current_dir().expect("Failed to get current dir");
    let ui_path = current_dir.join("vault_app/ui");

    let final_ui_path = if ui_path.exists() {
        ui_path.canonicalize().unwrap_or(ui_path)
    } else {
        PathBuf::from("ui")
    };

    println!("Starting Alakit Vault Showcase...");

    AlakitEngine::new("Alakit Vault Showcase")
        .with_assets::<Assets>()
        .with_ui_dir(final_ui_path.to_str().unwrap_or("ui"))
        .with_initial_url("index.html")
        .run();
}
