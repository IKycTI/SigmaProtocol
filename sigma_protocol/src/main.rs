use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Html,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use num_bigint::{BigUint, ToBigInt};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tracing::{info, warn};

use clap::Parser;

mod config;
mod key_gen;
mod math;

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
    q: BigUint,
    g: BigUint,
    h: BigUint,
    secret_key: Key,
    tx: broadcast::Sender<String>,
}

#[derive(Debug, Clone)]
struct Key {
    alpha: BigUint,
    beta: BigUint,
}

impl Key {
    fn new(alpha: BigUint, beta: BigUint) -> Self {
        Key { alpha, beta }
    }
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    tracing_subscriber::fmt::init();

    let (tx, _) = broadcast::channel::<String>(100);

    let module = key_gen::gen_random_prime().await;

    let state = AppState {
        config: match Config::load(&cli.config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        },
        q: module.clone(),
        g: key_gen::random_biguint_mod(&module).await,
        h: key_gen::random_biguint_mod(&module).await,
        secret_key: Key::new(
            key_gen::random_biguint_mod(&module).await,
            key_gen::random_biguint_mod(&module).await,
        ),
        tx,
    };

    let addr: SocketAddr = state.config.get_address().parse().unwrap();

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/start", post(start_handler))
        .route("/logs", get(logs_handler))
        .with_state(state);

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    info!("Listening on {}", addr);
}

async fn root_handler() -> Html<&'static str> {
    Html(include_str!("../html/index.html"))
}

async fn start_handler(State(state): State<AppState>) -> StatusCode {
    info!("Получен запрос на запуск задачи");

    // Клонируем sender — можно много раз
    let tx = state.tx.clone();

    tokio::spawn(async move {
        start_proof(state, tx);
        // simulate_long_task(tx).await;
    });

    StatusCode::ACCEPTED
}

async fn logs_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, axum::Error>>> {
    let stream = BroadcastStream::new(state.tx.subscribe()).map(|res| match res {
        Ok(msg) => Ok(Event::default().data(msg)),
        Err(BroadcastStreamRecvError::Lagged(skipped)) => {
            Ok(Event::default().data(format!("⚠️ Пропущено {} сообщений", skipped)))
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

//use std::time::Duration;

// async fn simulate_long_task(tx: broadcast::Sender<String>) {
//     // Отправка — игнорируем ошибки (если никто не слушает)
//     let _ = tx.send("🔧 Задача запущена".to_string());
//     tokio::time::sleep(Duration::from_millis(500)).await;

//     let steps = [
//         "📥 Получение данных...",
//         "⚙️ Обработка этап 1...",
//         "⚙️ Обработка этап 2...",
//         "💾 Сохранение результатов...",
//         "✅ Задача завершена успешно!",
//     ];

//     for &step in &steps {
//         let _ = tx.send(step.to_string());
//         tokio::time::sleep(Duration::from_millis(800)).await;
//     }

//     // Финальное сообщение
//     let _ = tx.send("🔚 Работа завершена".to_string());
// }

async fn start_proof(appstate: AppState, tx: broadcast::Sender<String>) {
    let c = get_challenge();

    let q = appstate.q.clone();
    let g = appstate.g.clone();
    let h = appstate.h.clone();
    let a = appstate.secret_key.alpha.clone();
    let b = appstate.secret_key.beta.clone();

    let at = key_gen::random_biguint_mod(&q).await;
    let bt = key_gen::random_biguint_mod(&q).await;
    let ut = (math::mod_pow_big(&g, &at.to_bigint().unwrap(), &q).unwrap()
        * math::mod_pow_big(&h, &bt.to_bigint().unwrap(), &q).unwrap())
        % &q;

    let az = (at + a * &c) % &q;
    let bz = (bt + b * &c) % &q;

    tx.send("P успешно вычислил и отправил значения az, bz, ut".to_string());
    send_proof(az, bz, ut, c, appstate.clone(), tx).await;
}

fn get_challenge() -> BigUint {
    todo!()
}

async fn send_proof(
    az: BigUint,
    bz: BigUint,
    ut: BigUint,
    c: BigUint,
    appstate: AppState,
    tx: broadcast::Sender<String>,
) {
    if true {
        let _ = tx.send("✅ Задача завершена успешно!".to_string());
    } else {
        let _ = tx.send("❌ Задача завершена с ошибкой!".to_string());
    }
}

async fn p_handler(State(state): State<AppState>) -> Result<&'static str, axum::http::StatusCode> {
    println!(
        "{} Hello it`s {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        state.config.get_name()
    );

    if true {
        return Ok("Good");
    } else {
        return Ok("Reject");
    }
}

// async fn fetch_handler(State(state): State<AppState>) -> Result<String, axum::http::StatusCode> {
//     let response = state
//         .http_client
//         .get(format!(
//             "http://{}",
//             state.config.get_second_server_address()
//         ))
//         .send()
//         .await
//         .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

//     let text = match response.text().await {
//         Ok(text) => text,
//         Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
//     };

//     Ok(text)
// }
