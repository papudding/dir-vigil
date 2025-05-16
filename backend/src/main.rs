use axum::{routing::get, Router};
use log::info;
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
mod config;
mod file_store;
mod handler;
mod logger;
mod totp;
mod util;
use clap::Parser;
use config::{AppState, Config};
use handler::{generate_captcha_handler, keep_alive_handler, status_handler};
use logger::init_logger;
use util::{check_remain_time, monitor_task};

pub const SECRET_FILE_NAME: &str = "user_secret.enc";
pub const SECRET_FILE_KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

#[tokio::main]
async fn main() {
    // 初始化日志
    init_logger();

    let config = Config::parse();
    if !std::path::Path::new(&config.directory).exists() {
        panic!("Directory does not exist: {}", config.directory);
    }

    let app_state = Arc::new(Mutex::new(AppState {
        captcha: None,
        last_active: Instant::now(),
        config: config.clone(),
        file_encypt_key: SECRET_FILE_KEY,
        serect_file_path: SECRET_FILE_NAME.to_string(),
    }));

    let secret = if std::path::Path::new(SECRET_FILE_NAME).exists() {
        file_store::decrypt_from_file(SECRET_FILE_KEY, SECRET_FILE_NAME)
            .expect("Failed to decrypt secret file")
    } else {
        let tmp_secrt = totp::generate_totp_secret();
        file_store::encrypt_and_save(&tmp_secrt, SECRET_FILE_KEY, SECRET_FILE_NAME)
            .expect("Failed to encrypt secret file");
        tmp_secrt
    };

    let uri = totp::generate_totp_uri(&secret, "dir-vigil", "myself");
    util::print_qrcode(&uri);
    info!("{}", uri);

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET])
        .allow_origin(Any)
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let router = Router::new()
        .route("/keepalive", get(keep_alive_handler))
        .route("/status", get(status_handler))
        .route("/captcha", get(generate_captcha_handler))
        .with_state(app_state.clone())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", app_state.lock().await.config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tokio::spawn(monitor_task(app_state.clone()));
    tokio::spawn(check_remain_time(app_state.clone(), config.clone()));

    info!("Server running on {}", addr);
    axum::serve(listener, router).await.unwrap();
}
