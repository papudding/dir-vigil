use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use log::info;
use std::{io::IsTerminal, path::Path, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

mod config;
mod file_store;
mod handler;
mod logger;
mod totp;
mod util;

use clap::Parser;
use config::{secret_key_from_env, AppState, Config};
use handler::{generate_captcha_handler, keep_alive_handler, status_handler};
use logger::init_logger;
use util::{check_remain_time, monitor_task};

pub const SECRET_FILE_NAME: &str = "user_secret.enc";

fn load_or_create_secret(secret_key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if Path::new(SECRET_FILE_NAME).exists() {
        file_store::decrypt_from_file(secret_key, SECRET_FILE_NAME)
            .map_err(|err| format!("failed to decrypt {SECRET_FILE_NAME}: {err}"))
    } else {
        let secret = totp::generate_totp_secret();
        file_store::encrypt_and_save(&secret, secret_key, SECRET_FILE_NAME)
            .map_err(|err| format!("failed to save {SECRET_FILE_NAME}: {err}"))?;
        info!("Generated a new TOTP secret in {SECRET_FILE_NAME}");
        Ok(secret)
    }
}

/// Print the enrollment QR code (interactive terminal) or otpauth URI (pipes/CI).
fn print_enrollment(secret: &[u8]) {
    let uri = totp::generate_totp_uri(secret, "dir-vigil", "myself");
    if std::io::stdout().is_terminal() {
        util::print_qrcode(&uri);
        eprintln!("Scan the QR code above with your TOTP app.");
    } else {
        // Explicit user request (`--print-secret-uri`): print the URI so it can be
        // forwarded into an authenticator on a non-interactive terminal.
        println!("{uri}");
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    // 初始化日志
    init_logger();

    let config = Config::parse();

    let secret_key = secret_key_from_env().unwrap_or_else(|err| {
        eprintln!("error: {err}");
        exit_with_error(
            "set DIR_VIGIL_SECRET_KEY and restart. For upgrades from <=0.1.0 use the previous \
             key; if it is unknown, delete user_secret.enc and scan a new QR code",
        );
    });

    // First-time setup helper: print the enrollment code and exit.
    if config.print_secret_uri {
        let secret = load_or_create_secret(&secret_key).unwrap_or_else(|err| exit_with_error(&err));
        print_enrollment(&secret);
        return;
    }

    let Some(directory) = config.directory.clone() else {
        exit_with_error("--directory is required unless --print-secret-uri is used");
    };

    if !Path::new(&directory).exists() {
        exit_with_error(&format!("directory does not exist: {directory}"));
    }

    info!(
        "Starting dir-vigil: directory={} host={} port={} timeout={}s warning={}s checking={}s",
        directory,
        config.host,
        config.port,
        config.timeout_seconds,
        config.warning_seconds,
        config.checking_interval
    );

    let app_state = Arc::new(Mutex::new(AppState {
        captcha: None,
        captcha_attempts: 0,
        last_active: Instant::now(),
        config: config.clone(),
        file_encypt_key: secret_key,
        serect_file_path: SECRET_FILE_NAME.to_string(),
    }));

    let secret = load_or_create_secret(&secret_key).unwrap_or_else(|err| exit_with_error(&err));

    let uri = totp::generate_totp_uri(&secret, "dir-vigil", "myself");
    if std::io::stdout().is_terminal() {
        util::print_qrcode(&uri);
        eprintln!("Scan the QR code above with your TOTP app.");
    } else {
        // Never write the secret/URI into logs: retrieve it explicitly with
        // `dir-vigil --print-secret-uri` when needed.
        info!(
            "TOTP secret loaded from {SECRET_FILE_NAME}; run `dir-vigil --print-secret-uri` to show the QR code"
        );
    }

    let mut router = Router::new()
        .route("/keepalive", post(keep_alive_handler))
        .route("/status", get(status_handler))
        .route("/captcha", get(generate_captcha_handler))
        .with_state(app_state.clone());

    if let Some(origin) = &config.cors_origin {
        let origin_header = origin
            .parse::<HeaderValue>()
            .unwrap_or_else(|_| exit_with_error(&format!("invalid --cors-origin: {origin}")));
        router = router.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(origin_header)
                .allow_headers([header::CONTENT_TYPE]),
        );
        info!("CORS enabled for origin {origin}");
    }

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| exit_with_error(&format!("failed to bind {addr}: {err}")));

    tokio::spawn(monitor_task(app_state.clone()));
    tokio::spawn(check_remain_time(app_state.clone(), config.clone()));

    info!("Server running on {addr}");
    axum::serve(listener, router).await.unwrap();
}
