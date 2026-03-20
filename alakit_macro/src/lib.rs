use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};

/// Custom attribute macro that registers the specified struct into the central
/// AlakitController registry (inventory) under the given namespace.
#[proc_macro_attribute]
pub fn alakit_controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Extract namespace
    let namespace = parse_macro_input!(attr as LitStr);

    // Parse the target struct
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Generate original definition + inventory registration
    let expanded = quote! {
        #input

        // Inventory submit block for dynamic registration
        inventory::submit! {
            alakit::ControllerRegistration {
                namespace: #namespace,
                factory: || Box::new(#struct_name::default()) as Box<dyn alakit::AlakitController + Send + Sync>,
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute for main.rs: dynamic controller discovery.
/// Replaces the legacy build.rs and mod.rs mechanism.
#[proc_macro_attribute]
pub fn alakit_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let controllers_dir = std::path::Path::new(&manifest_dir).join("src/controllers");

    let mut mod_decls = String::new();
    if controllers_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(controllers_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let name = path.file_stem().unwrap().to_string_lossy();
                    if name != "mod" {
                        // Double backslash for Windows absolute path recognition
                        let safe_path = path.to_string_lossy().replace("\\", "\\\\");
                        mod_decls.push_str(&format!("#[path = \"{}\"]\npub mod {};\n", safe_path, name));
                    }
                }
            }
        }
    }

    let mod_tokens: proc_macro2::TokenStream = mod_decls.parse().unwrap();
    
    let expanded = quote! {
        pub mod controllers {
            #mod_tokens
        }

        #input
    };

    TokenStream::from(expanded)
}

/// Encrypted embedding of UI files (XOR / AES obfuscation)
#[proc_macro]
pub fn alakit_assets(input: TokenStream) -> TokenStream {
    let folder_path_lit = parse_macro_input!(input as LitStr);
    let folder_path = folder_path_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::path::Path::new(&manifest_dir).join(&folder_path);

    let mut files = Vec::new();
    collect_files(&base_dir, "", &mut files);

    // Generate encryption key at compile time
    let mut key = [0u8; 32];
    let start = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    let mut seed = start;
    for i in 0..32 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        key[i] = (seed >> 32) as u8;
    }

    let key_tokens = key.iter().map(|b| quote!(#b));

    let mut match_arms = proc_macro2::TokenStream::new();

    for (name, content) in files {
        let mut encrypted = content.clone();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= key[i % 32];
        }

        let byte_str = proc_macro2::Literal::byte_string(&encrypted);

        match_arms.extend(quote! {
            #name => {
                let encrypted = #byte_str;
                let mut decrypted = Vec::with_capacity(encrypted.len());
                for (i, &byte) in encrypted.iter().enumerate() {
                    decrypted.push(byte ^ KEY[i % 32]);
                }
                Some(std::borrow::Cow::Owned(decrypted))
            },
        });
    }

    let expanded = quote! {
        {
            struct AlakitEmbeddedAssets;
            impl alakit::engine::AssetProvider for AlakitEmbeddedAssets {
                fn get(&self, path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
                    const KEY: [u8; 32] = [ #(#key_tokens),* ];
                    match path {
                        #match_arms
                        _ => None,
                    }
                }
            }
            std::sync::Arc::new(AlakitEmbeddedAssets)
        }
    };

    TokenStream::from(expanded)
}

fn collect_files(dir: &std::path::Path, prefix: &str, files: &mut Vec<(String, Vec<u8>)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let mut name = if prefix.is_empty() {
                entry.file_name().to_string_lossy().to_string()
            } else {
                format!("{}/{}", prefix, entry.file_name().to_string_lossy())
            };
            name = name.replace("\\", "/");

            if path.is_dir() {
                collect_files(&path, &name, files);
            } else if path.is_file() {
                if let Ok(content) = std::fs::read(&path) {
                    files.push((name, content));
                }
            }
        }
    }
}
