use axum::{Router, extract::State, routing::get};
use clap::Parser;
use std::net::SocketAddr;

mod config;
use config::Config;

// const PATH: &str = "config_p.json";

#[derive(Parser)]
struct Args {
    /// Путь до конфигурации сервера
    #[arg(short, long)]
    config_path: String,
}

#[derive(Debug, Clone)]
struct AppState {
    config: Config,
    http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();

    let state = AppState {
        config: match Config::load(&cli.config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        },
        http_client: reqwest::Client::new(),
    };

    let addr: SocketAddr = state.config.get_address().parse().unwrap();

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/fetch", get(fetch_handler))
        .with_state(state);

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn root_handler(State(state): State<AppState>) -> Result<(), axum::http::StatusCode> {
    println!("Hello it`s {}", state.config.get_name());
    let result = fetch_handler(State(state.clone())).await?;
    Ok(())
}

async fn fetch_handler(State(state): State<AppState>) -> Result<String, axum::http::StatusCode> {
    let response = state
        .http_client
        .get(format!(
            "http://{}",
            state.config.get_second_server_address()
        ))
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let text = response
        .text()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(text)
}
