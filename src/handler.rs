use crate::config::AppState;
use crate::file_store;
use crate::totp;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

pub async fn keep_alive_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<Vec<(String, String)>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    if let Some((_, code)) = params.iter().find(|(k, _)| k == "tow_fa_code") {
        let secret = file_store::decrypt_from_file(state.file_encypt_key, &state.serect_file_path)
            .expect("Failed to decrypt secret file");
        if totp::verify_totp_code(
            &secret,
            code.parse().unwrap(),
            30,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ) {
            state.last_active = Instant::now();
            return (StatusCode::OK, "Status updated");
        }
    }

    (StatusCode::FORBIDDEN, "Invalid 2FA code")
}

pub async fn status_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let elapsed = state.last_active.elapsed();
    let remaining = Duration::from_secs(state.config.timeout_seconds).saturating_sub(elapsed);
    format!(
        "Remaining time: {} seconds\nDirectory: {}",
        remaining.as_secs(),
        state.config.directory
    )
}
