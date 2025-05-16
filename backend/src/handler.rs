use crate::config::AppState;
use crate::file_store;
use crate::totp;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;
use rand::{rng, Rng};
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

pub async fn generate_captcha_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    let captcha: String = rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    state.captcha = Some(captcha.clone());
    info!("Generated captcha: {}", captcha);
    (StatusCode::OK, captcha)
}

async fn validate_captcha(state: &mut AppState, captcha_code: &str) -> Result<(), (StatusCode, &'static str)> {
    match &state.captcha {
        Some(stored_captcha) if captcha_code.to_ascii_lowercase() == stored_captcha.to_ascii_lowercase() => Ok(()),
        Some(_) => Err((StatusCode::FORBIDDEN, "Incorrect captcha")),
        None => Err((StatusCode::FORBIDDEN, "No captcha generated")),
    }
}

async fn validate_2fa(state: &mut AppState, code: &str) -> Result<(), (StatusCode, &'static str)> {
    let secret = file_store::decrypt_from_file(
        state.file_encypt_key,
        &state.serect_file_path,
    )
    .expect("Failed to decrypt secret file");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if totp::verify_totp_code(&secret, code.parse().unwrap(), 30, timestamp) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Invalid 2FA code"))
    }
}

pub async fn keep_alive_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<Vec<(String, String)>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    let captcha_code = match params.iter().find(|(k, _)| k == "captcha") {
        Some((_, code)) => code,
        None => return (StatusCode::FORBIDDEN, "No captcha code provided"),
    };

    if let Err((status, message)) = validate_captcha(&mut state, captcha_code).await {
        return (status, message);
    }

    let tow_fa_code = match params.iter().find(|(k, _)| k == "tow_fa_code") {
        Some((_, code)) => code,
        None => return (StatusCode::FORBIDDEN, "No 2FA code provided"),
    };

    if let Err((status, message)) = validate_2fa(&mut state, tow_fa_code).await {
        return (status, message);
    }

    state.last_active = Instant::now();
    (StatusCode::OK, "Status updated")
}

pub async fn status_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let elapsed = state.last_active.elapsed();
    let comment = state.config.comment.clone().unwrap_or(String::from("^-^"));
    let remaining = Duration::from_secs(state.config.timeout_seconds).saturating_sub(elapsed);
    format!(
        "{{ \"remainsSeconds\": {}, \"comment\": \"{}\"}}",
        remaining.as_secs(),
        comment
    )
}
