mod config;
mod db;
mod dtos;
mod error;
mod handler;
mod middleware;
mod models;
mod router;
mod utils;

use std::sync::Arc;

use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
};
use config::Config;
use db::{DBClient, UserExt};
use dotenv::dotenv;
use router::create_router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[tokio::main]
async fn main() {
    println!("🚀 MAIN STARTED");
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();
    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Connected to the database successfully.");
            pool
        }
        Err(e) => {
            println!("Failed to connect to the database: {:?}", e);
            std::process::exit(1);
        }
    };

   let cors = CorsLayer::new()
    .allow_origin([
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "https://your-frontend-domain.vercel.app"
            .parse::<HeaderValue>()
            .unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT, COOKIE])
    .allow_credentials(true);

    let db_client = DBClient::new(pool);
    let app_state = Arc::new(AppState {
        env: config.clone(),
        db_client: db_client.clone(),
    });

    let app = create_router(app_state.clone()).layer(cors.clone());

    println!(
        "{}",
        format!(" Server is running on http://localhost:{}", config.port)
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
