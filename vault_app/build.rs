use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // --- UI Asset Bundling Configuration ---
    // Ha a felhasználó futtatáskor exportálta az ALAKIT_UI_DIR változót, azt használjuk,
    // különben az alapértelmezett "ui/" könyvtárat.
    let ui_dir = env::var("ALAKIT_UI_DIR").unwrap_or_else(|_| "ui/".to_string());

    // Átadjuk ezt a változót a rust-embed makrónak fordítási időben
    println!("cargo:rustc-env=ALAKIT_UI_DIR={}", ui_dir);
    // Újrafordítjuk a kódot, ha ez a környezeti változó megváltozik
    println!("cargo:rerun-if-env-changed=ALAKIT_UI_DIR");

    // --- Controller Auto-Discovery ---
    let controllers_dir_env =
        env::var("ALAKIT_CONTROLLERS_DIR").unwrap_or_else(|_| "src/controllers".to_string());
    println!("cargo:rerun-if-env-changed=ALAKIT_CONTROLLERS_DIR");
    println!("cargo:rerun-if-changed={}", controllers_dir_env);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_controllers.rs");

    let mut mod_content = String::new();
    let controllers_dir = Path::new(&controllers_dir_env);

    if controllers_dir.exists() {
        // Canonicalize abszolút path megszerzése a precíz include-okhoz
        if let Ok(abs_dir) = controllers_dir.canonicalize() {
            if let Ok(entries) = fs::read_dir(&abs_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "rs" {
                                    if let Some(file_name) = path.file_stem() {
                                        let name = file_name.to_string_lossy();
                                        if name != "mod" {
                                            // Escape the absolute path backslashes for macro inclusion
                                            let safe_path =
                                                path.to_string_lossy().replace("\\", "/");
                                            mod_content.push_str(&format!(
                                                "#[path = \"{}\"]\n",
                                                safe_path
                                            ));
                                            mod_content.push_str(&format!("pub mod {};\n", name));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fs::write(&dest_path, mod_content).unwrap();
}
