use std::{
    borrow::Cow,
    path::PathBuf,
    sync::Arc,
};
#[cfg(debug_assertions)]
use std::fs;

use crate::{AppContext, RustDOM};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebViewBuilder, http::Response};

/// Trait az aszettek (HTML, CSS, JS) eléréséhez.
/// Release módban ez biztosítja a binárisba ágyazott fájlok elérését.
pub trait AssetProvider: Send + Sync + 'static {
    fn get(&self, path: &str) -> Option<Cow<'static, [u8]>>;
}

/// Alapértelmezett üres provider
pub struct NoAssets;
impl AssetProvider for NoAssets {
    fn get(&self, _path: &str) -> Option<Cow<'static, [u8]>> {
        None
    }
}

pub struct AlakitEngine {
    title: String,
    ui_dir: PathBuf,
    initial_url: String,
    asset_provider: Arc<dyn AssetProvider>,
}

impl AlakitEngine {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ui_dir: PathBuf::from("ui"),
            initial_url: "index.html".to_string(),
            asset_provider: Arc::new(NoAssets),
        }
    }

    /// Beállítja a beágyazott aszetteket szolgáltató típust.
    pub fn with_assets<T: rust_embed::RustEmbed + Send + Sync + 'static>(mut self) -> Self {
        struct EmbedProvider<T>(std::marker::PhantomData<T>);
        impl<T: rust_embed::RustEmbed + Send + Sync + 'static> AssetProvider for EmbedProvider<T> {
            fn get(&self, path: &str) -> Option<Cow<'static, [u8]>> {
                T::get(path).map(|f| f.data)
            }
        }
        self.asset_provider = Arc::new(EmbedProvider::<T>(std::marker::PhantomData));
        self
    }

    pub fn with_ui_dir(mut self, path: &str) -> Self {
        self.ui_dir = PathBuf::from(path);
        self
    }

    pub fn with_initial_url(mut self, url: &str) -> Self {
        self.initial_url = url.trim_start_matches('/').to_string();
        self
    }

    pub fn run(self) {
        let event_loop = EventLoopBuilder::<String>::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop)
            .unwrap();

        // UI mappa rögzítése.
        let _final_ui_dir = self.ui_dir.canonicalize().unwrap_or(self.ui_dir.clone());

        const CORE_JS: &str = include_str!("alakit-core.js");
        const CORE_CSS: &str = include_str!("alakit-core.css");

        let proxy = event_loop.create_proxy();
        let global_store = crate::store::Store::new(proxy.clone());
        let _asset_provider = self.asset_provider.clone();

        #[cfg(debug_assertions)]
        let ui_dir_clone = _final_ui_dir.clone();

        // CSS Injector script
        let css_injection_script = format!(
            r#"
            (function() {{
                var inject = function() {{
                    if (document.head) {{
                        if (document.getElementById('alakit-core-styles')) return;
                        var style = document.createElement('style');
                        style.id = 'alakit-core-styles';
                        style.textContent = `{}`;
                        document.head.appendChild(style);
                    }} else {{
                        setTimeout(inject, 10);
                    }}
                }};
                inject();
            }})();
            "#,
            CORE_CSS.replace('`', "\\`").replace('$', "\\$")
        );

        let webview = WebViewBuilder::new()
            .with_devtools(cfg!(debug_assertions))
            .with_custom_protocol("alakit".into(), move |_webview_id, request| {
                let full_uri = request.uri().to_string();

                // Asset path kinyerése a protokolból
                let asset_path_str = if full_uri.contains("localhost") {
                    let parts: Vec<&str> = full_uri.split("localhost").collect();
                    parts
                        .last()
                        .unwrap_or(&"")
                        .trim_start_matches('/')
                        .to_string()
                } else {
                    full_uri
                        .replace("alakit://", "")
                        .trim_start_matches('/')
                        .to_string()
                };

                let final_asset = if asset_path_str.is_empty() || asset_path_str == "/" {
                    "index.html".to_string()
                } else {
                    asset_path_str
                };

                #[cfg(debug_assertions)]
                {
                    let target_file = ui_dir_clone.join(&final_asset);

                    if target_file.is_file() {
                        let content = fs::read(&target_file).unwrap_or_default();
                        let mime_type = mime_guess::from_path(&target_file)
                            .first_or_octet_stream()
                            .to_string();

                        return create_response(content, &mime_type).unwrap();
                    } else {
                        return create_response(b"404 Not Found".to_vec(), "text/plain").unwrap();
                    }
                }

                #[cfg(not(debug_assertions))]
                {
                    match _asset_provider.get(&final_asset) {
                        Some(content) => {
                            let mime_type = mime_guess::from_path(&final_asset)
                                .first_or_octet_stream()
                                .to_string();
                            return create_response(content.into_owned(), &mime_type).unwrap();
                        }
                        None => {
                            return create_response(b"404 Not Found".to_vec(), "text/plain")
                                .unwrap();
                        }
                    }
                }
            })
            .with_ipc_handler(move |req: wry::http::Request<String>| {
                let message = req.body();
                let proxy_inner = proxy.clone();
                let store_inner = global_store.clone();

                if let Some((controller, rest)) = message.split_once(':') {
                    let (command, args) = match rest.split_once('|') {
                        Some((cmd, a)) => (cmd, a),
                        None => (rest, ""),
                    };

                    // --- BELSŐ ALAKIT PARANCSOK KEZELÉSE ---
                    if controller == "alakit" {
                        match command {
                            "log" => {
                                if let Ok(log_data) =
                                    serde_json::from_str::<serde_json::Value>(args)
                                {
                                    let level = log_data["level"].as_str().unwrap_or("info");
                                    let msg = log_data["msg"].as_str().unwrap_or("");
                                    match level {
                                        "error" => println!("🔴 [JS ERROR] {}", msg),
                                        "warn" => println!("🟡 [JS WARN]  {}", msg),
                                        _ => println!("🌐 [JS LOG]   {}", msg),
                                    }
                                }
                            }
                            "init" => {
                                if let Ok(init_data) =
                                    serde_json::from_str::<serde_json::Value>(args)
                                {
                                    let key = init_data["key"].as_str().unwrap_or("");
                                    let val = init_data["val"].as_str().unwrap_or("");
                                    if !key.is_empty() && store_inner.get(key).is_none() {
                                        store_inner.set(key, val);
                                    }
                                }
                            }
                            _ => println!("⚠️ [ALAKIT] Ismeretlen belső parancs: {}", command),
                        }
                        return;
                    }

                    let ctx = AppContext {
                        dom: RustDOM { proxy: proxy_inner },
                        store: store_inner,
                    };

                    let mut _handled = false;
                    for reg in inventory::iter::<crate::core::ControllerRegistration> {
                        if reg.namespace == controller {
                            let controller_instance = (reg.factory)();
                            controller_instance.handle(command, args, &ctx);
                            _handled = true;
                            break;
                        }
                    }
                }
            })
            .with_initialization_script(&format!("{}\n{}", CORE_JS, css_injection_script))
            .with_url(&format!("alakit://localhost/{}", self.initial_url))
            .build(&window)
            .expect("Failed to build webview");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::UserEvent(js_code) => {
                    let _ = webview.evaluate_script(&js_code);
                }
                _ => {}
            }
        });
    }
}

fn create_response(
    body: Vec<u8>,
    mime_type: &str,
) -> wry::http::Result<Response<Cow<'static, [u8]>>> {
    Response::builder()
        .header("Content-Type", mime_type)
        .header("Access-Control-Allow-Origin", "*")
        .body(Cow::Owned(body))
}
