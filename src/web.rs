use anyhow::Result;
use axum::{
    response::Html,
    routing::get,
    Json,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    application: String,
}

async fn home() -> Html<String> {
    let html = std::fs::read_to_string("templates/index.html")
        .unwrap_or_else(|_| {
            "<h1>Impossible de charger la page d'accueil</h1>".to_string()
        });

    Html(html)
}

async fn dashboard() -> Html<String> {
    let html = std::fs::read_to_string("templates/dashboard.html")
        .unwrap_or_else(|_| {
            "<h1>Impossible de charger le dashboard</h1>".to_string()
        });

    Html(html)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        application: "ADP Trafic aérien".to_string(),
    })
}

pub async fn start_server() -> Result<()> {
    let app = Router::new()
        .route("/", get(home))
        .route("/dashboard", get(dashboard))
        .route("/api/health", get(health))
        .nest_service("/static", ServeDir::new("static"));

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Accueil : http://127.0.0.1:3000");
    println!("Dashboard : http://127.0.0.1:3000/dashboard");
    println!("API : http://127.0.0.1:3000/api/health");

    let listener = tokio::net::TcpListener::bind(address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}