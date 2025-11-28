use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Html,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use num_bigint::{BigInt, BigUint, ToBigInt};
use std::{
    io::Error,
    net::{SocketAddr, ToSocketAddrs},
};
use tokio::sync::{broadcast, watch::error::SendError};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tracing::{info, warn};

use clap::Parser;
use std::time::Duration;

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
    tx: broadcast::Sender<String>,
}

impl AppState {
    async fn new(config_path: String) -> Self {
        let config = match Config::load(&config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        };

        let (tx, _) = broadcast::channel::<String>(100);

        let module = key_gen::gen_random_prime().await;

        let state = AppState {
            config,
            q: module.clone(),
            g: match key_gen::generated_element(&module).await {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Failed to generate element: {}", e);
                    std::process::exit(1);
                }
            },
            h: match key_gen::generated_element(&module).await {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Failed to generate element: {}", e);
                    std::process::exit(1);
                }
            },
            tx,
        };
        state
    }

    async fn get_challenge(&self) -> BigUint {
        let c = key_gen::random_biguint_mod(&self.q).await;
        let _ = self.tx.send(format!("Hello I`m Victor. {}", c));
        tokio::time::sleep(Duration::from_millis(500)).await;
        info!("V сгенерировал с");
        c
    }
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

async fn compute_u(key: &Key, g: &BigUint, h: &BigUint, q: &BigUint) -> BigUint {
    let a = match key.alpha.to_bigint() {
        Some(a) => a,
        None => {
            warn!("Failed to convert alpha to bigint");
            std::process::exit(1);
        }
    };
    let b = match key.beta.to_bigint() {
        Some(b) => b,
        None => {
            warn!("Failed to convert beta to bigint");
            std::process::exit(1);
        }
    };

    (math::mod_pow_big(g, &a, &q).unwrap() * math::mod_pow_big(h, &b, &q).unwrap()) % q
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    tracing_subscriber::fmt::init();

    let state = AppState::new(cli.config_path).await;

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
    info!("Получен запрос на запуск задач");

    let tx = state.tx.clone();
    while tx.receiver_count() == 0 {
        warn!("Receivers count equal 0. Wait");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    tokio::spawn(async move {
        start_proof(state, tx).await;
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

async fn start_proof(appstate: AppState, tx: broadcast::Sender<String>) {
    info!("Начинаем проверку");
    let q = &appstate.q;

    let secret_key = Key::new(
        key_gen::random_biguint_mod(&q).await,
        key_gen::random_biguint_mod(&q).await,
    );
    info!("P Сгенерировал альфа и бета");
    let u = compute_u(&secret_key, &appstate.g, &appstate.h, &appstate.q).await;

    info!("P Вычислил публичный ключ");
    let _ = tx.send(format!(
        "Hi, I'm Pavel! And I know the secret key! here is my public key: {} \n secret key ({}, {})",
        u, secret_key.alpha, secret_key.beta
    ))
    .inspect_err(|e| warn!("Error log stream: {}", e));

    tokio::time::sleep(Duration::from_millis(500)).await;

    let g = &appstate.g;
    let h = &appstate.h;

    let keyt = Key::new(
        key_gen::random_biguint_mod(&q).await,
        key_gen::random_biguint_mod(&q).await,
    );

    info!("P Сгенерировал альфа_t и бета_t");
    let ut = compute_u(&keyt, g, h, q).await;

    info!("P Вычислил u_t");
    let c = appstate.get_challenge().await;

    info!("P Получили испытание!");

    let keyz = Key::new(
        (keyt.alpha + secret_key.alpha * &c) % q,
        (keyt.beta + secret_key.beta * &c) % q,
    );

    info!("P Вычислил альфа_z и бета_z");

    let _ = tx
        .send("P успешно вычислил и отправил значения az, bz, ut".to_string())
        .inspect_err(|e| warn!("Error log stream: {}", e));
    tokio::time::sleep(Duration::from_millis(500)).await;
    send_proof(keyz, u, ut, c.to_bigint().unwrap(), appstate.clone(), tx).await;
}

async fn send_proof(
    key: Key,
    u: BigUint,
    ut: BigUint,
    c: BigInt,
    appstate: AppState,
    tx: broadcast::Sender<String>,
) {
    let uz = compute_u(&key, &appstate.g, &appstate.h, &appstate.q).await;

    info!("V вычислил u_z");
    let uc = match math::mod_pow_big(&u, &c, &appstate.q) {
        Some(u) => u,
        None => {
            let _ = tx
                .send("Задача завершена с ошибкой!".to_string())
                .inspect_err(|e| warn!("Error log stream: {}", e));
            return;
        }
    };
    let utuc = ut * uc % &appstate.q;
    info!("V вычислил u_t * u^z");
    if uz == utuc {
        info!("V подтверлил знание");
        let _ = tx
            .send("Доступ разрешен!".to_string())
            .inspect_err(|e| warn!("Error log stream: {}", e));
    } else {
        info!("V отверг знание");
        let _ = tx
            .send("В доступе отказано!".to_string())
            .inspect_err(|e| warn!("Error log stream: {}", e));
    }
}
