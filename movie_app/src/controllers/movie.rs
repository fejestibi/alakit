use alakit::{AlakitController, AppContext};
use alakit_macro::alakit_controller;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TVMazeShow {
    _id: u32,
    name: String,
    genres: Vec<String>,
    rating: Option<Rating>,
    image: Option<Image>,
    _summary: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Rating {
    average: Option<f32>,
}

#[derive(Deserialize, Debug)]
struct Image {
    medium: String,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    show: TVMazeShow,
}

#[alakit_controller("movie")]
#[derive(Default)]
pub struct MovieController;

impl AlakitController for MovieController {
    fn handle(&self, command: &str, args: &str, ctx: &AppContext) {
        match command {
            "search" => {
                let mut query = args.to_string();
                let store = ctx.store.clone();
                let dom = ctx.dom.clone();
                
                // If source is alakit-form (JSON), extract the 'search' field
                if query.starts_with('{') {
                    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&query) {
                        query = json_data["search"].as_str().unwrap_or(&query).to_string();
                    }
                }

                if query.trim().is_empty() {
                    dom.toast_info("Please enter a search term!");
                    return;
                }

                dom.log(&format!("Starting search for: {}", query));

                // Start async call with Tokio
                tokio::spawn(async move {
                    let url = format!("https://api.tvmaze.com/search/shows?q={}", query);
                    
                    match reqwest::get(&url).await {
                        Ok(response) => {
                            if let Ok(results) = response.json::<Vec<SearchResult>>().await {
                                let mut html = String::new();
                                
                                if results.is_empty() {
                                    html = r#"<div class="empty-state"><p>No results found for your search.</p></div>"#.to_string();
                                } else {
                                    for item in results {
                                        let show = item.show;
                                        let rating = show.rating.and_then(|r| r.average).map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string());
                                        let genres = show.genres.join(", ");
                                        let poster = show.image.map(|i| format!(r#"<img src="{}" alt="{}">"#, i.medium, show.name))
                                            .unwrap_or_else(|| r#"<div class="no-poster">No poster</div>"#.to_string());
                                        
                                        html.push_str(&format!(r#"
                                            <div class="movie-card">
                                                <div class="poster-wrapper">
                                                    {}
                                                </div>
                                                <div class="movie-info">
                                                    <h3>{}</h3>
                                                    <div class="rating">⭐ {}</div>
                                                    <div class="genres">{}</div>
                                                </div>
                                            </div>
                                        "#, poster, show.name, rating, genres));
                                    }
                                }
                                
                                store.set("results", &html);
                            }
                        },
                        Err(e) => {
                            println!("API Error: {}", e);
                        }
                    }
                });
            },
            _ => println!("Unknown movie command: {}", command),
        }
    }
}
