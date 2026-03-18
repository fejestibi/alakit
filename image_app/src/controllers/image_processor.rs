use alakit::core::{AlakitController, AppContext};
use alakit_macro::alakit_controller;
use image::{imageops, DynamicImage, RgbaImage};
use rfd::FileDialog;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use std::sync::Mutex;

lazy_static::lazy_static! {
    // Memóriában tároljuk az aktuálisan nyitott képet
    static ref CURRENT_IMAGE: Mutex<Option<DynamicImage>> = Mutex::new(None);
}

#[alakit_controller("image")]
#[derive(Default)]
pub struct ImageController;

impl ImageController {
    // Belső segédfüggvény a DynamicImage Base64 Jpeg-be konvertálására és DOM injektálására
    fn render_image_to_dom(&self, img: &DynamicImage, ctx: &AppContext, message: &str) {
        let mut jpeg_data: Vec<u8> = Vec::new();
        let start_time = std::time::Instant::now();
        
        if let Ok(_) = img.write_to(&mut Cursor::new(&mut jpeg_data), image::ImageFormat::Jpeg) {
            let base64_img = general_purpose::STANDARD.encode(&jpeg_data);
            let src = format!("data:image/jpeg;base64,{}", base64_img);
            
            ctx.dom.get_id("preview_step").set_style("display", "flex");
            ctx.dom.get_id("filter_buttons").set_style("display", "flex");
            ctx.dom.get_id("watermark_tool").set_style("display", "block");
            ctx.dom.get_id("image_view").set_attribute("src", &src);
            
            let elapsed = start_time.elapsed().as_millis();
            ctx.dom.get_id("status_text").set_text(&format!("{} (Feldolgozva {}ms alatt)", message, elapsed));
            ctx.dom.toast_success("Kép sikeresen renderelve!");
        } else {
            ctx.dom.toast_error("Hiba a kép Jpeg kódolása közben!");
        }
    }
}

#[async_trait::async_trait]
impl AlakitController for ImageController {
    async fn handle(&self, command: &str, args: &str, ctx: AppContext) {
        match command {
            "open" => {
                ctx.dom.toast_info("Dilektálási párbeszédpanel megnyitva...");
                let file = FileDialog::new()
                    .add_filter("Képek", &["png", "jpg", "jpeg", "webp"])
                    .set_title("Válassz egy képet")
                    .pick_file();

                if let Some(path) = file {
                    match image::open(&path) {
                        Ok(img) => {
                            let mut current = CURRENT_IMAGE.lock().unwrap();
                            *current = Some(img.clone());
                            
                            self.render_image_to_dom(&img, &ctx, "Kép sikeresen betöltve a lemezről.");
                        }
                        Err(e) => {
                            ctx.dom.toast_error(&format!("Nem sikerült megnyitni a képet: {}", e));
                        }
                    }
                } else {
                    ctx.dom.toast_info("Képválasztás megszakítva.");
                }
            }
            "filter" => {
                let mut current = CURRENT_IMAGE.lock().unwrap();
                if let Some(img) = current.as_mut() {
                    let mut processed_img: DynamicImage = img.clone();
                    ctx.dom.log(&format!("Filter kérés érkezett: '{}'", args));
                    match args.trim() {
                        "grayscale" => {
                            processed_img = processed_img.grayscale();
                        }
                        "invert" => {
                            processed_img.invert();
                        }
                        "blur" => {
                            processed_img = processed_img.blur(3.0);
                        }
                        _ => {
                            ctx.dom.toast_error(&format!("Ismeretlen filter: '{}'", args.trim()));
                            return;
                        }
                    }
                    
                    self.render_image_to_dom(&processed_img, &ctx, &format!("Filter ({}): Kész", args));
                    *current = Some(processed_img); // Eredmény mentése további módosításokhoz
                } else {
                    ctx.dom.toast_error("Nincs megnyitott kép!");
                }
            }
            _ => {}
        }
    }

    async fn handle_binary(&self, command: &str, payload: &[u8], ctx: AppContext) {
        if command != "watermark" {
            return;
        }

        if payload.len() < 4 {
            ctx.dom.toast_error("Hibás IPC csomagméret!");
            return;
        }

        let header_len = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        
        if payload.len() < 4 + header_len {
            ctx.dom.toast_error("Csonkított Header adat!");
            return;
        }

        let header_json = std::str::from_utf8(&payload[4..4 + header_len]).unwrap_or("{}");
        let header: serde_json::Value = match serde_json::from_str(header_json) {
            Ok(h) => h,
            Err(_) => {
                ctx.dom.toast_error("Hibás fejléc JSON formátum!");
                return;
            }
        };

        let w = header["w"].as_u64().unwrap_or(0) as u32;
        let h = header["h"].as_u64().unwrap_or(0) as u32;
        let pixels = &payload[4 + header_len..];

        ctx.dom.log(&format!("Rust megkapta a Vízjelet: {}x{} - Px méret: {}", w, h, pixels.len()));

        let mut current = CURRENT_IMAGE.lock().unwrap();
        if let Some(img) = current.as_mut() {
            if let Some(watermark) = RgbaImage::from_raw(w, h, pixels.to_vec()) {
                let mut processed_img = img.clone();
                let watermark_dynamic = DynamicImage::ImageRgba8(watermark);
                
                // Középre pozícionáljuk a vízjelet (vagy felülre)
                let x = (processed_img.width().saturating_sub(w)) / 2;
                let y = (processed_img.height().saturating_sub(h)) / 2;

                imageops::overlay(&mut processed_img, &watermark_dynamic, x as i64, y as i64);

                self.render_image_to_dom(&processed_img, &ctx, "Vízjel sikeresen rásütve!");
                *current = Some(processed_img);
            } else {
                 ctx.dom.toast_error("Érvénytelen nyers pixel adat a vízjelhez!");
            }
        } else {
            ctx.dom.toast_error("Nincs megnyitott kép, amin alkalmazhatnám a vízjelet!");
        }
    }
}
