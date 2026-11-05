use crate::config::AppState;
use crate::file_store;
use crate::totp;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::debug;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

/// A captcha can be tried a few times (typos), but not brute-forced.
const MAX_CAPTCHA_ATTEMPTS: u32 = 5;

/// Returned when the TOTP code is wrong. The captcha is consumed at the same
/// time, so the caller must request a new one.
const INVALID_2FA_CAPTCHA_MESSAGE: &str = "Invalid 2FA code, request a new captcha";

#[derive(Deserialize)]
pub struct KeepAliveRequest {
    pub captcha: String,
    #[serde(rename = "tow_fa_code")]
    pub two_fa_code: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    #[serde(rename = "remainsSeconds")]
    pub remains_seconds: u64,
    pub comment: String,
}

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
    state.captcha_attempts = 0;
    debug!("Generated a new captcha");
    (StatusCode::OK, captcha)
}

fn captcha_matches(stored: &str, presented: &str) -> bool {
    presented.eq_ignore_ascii_case(stored)
}

fn validate_2fa(state: &AppState, code: &str) -> Result<(), (StatusCode, &'static str)> {
    let secret = file_store::decrypt_from_file(&state.file_encypt_key, &state.serect_file_path)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read TOTP secret",
            )
        })?;

    let code = code
        .parse::<u32>()
        .map_err(|_| (StatusCode::FORBIDDEN, "Invalid 2FA code"))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if totp::verify_totp_code(&secret, code, 30, timestamp) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Invalid 2FA code"))
    }
}

pub async fn keep_alive_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<KeepAliveRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    let Some(stored_captcha) = state.captcha.clone() else {
        return (StatusCode::FORBIDDEN, "No captcha generated");
    };

    state.captcha_attempts += 1;
    if state.captcha_attempts > MAX_CAPTCHA_ATTEMPTS {
        state.captcha = None;
        state.captcha_attempts = 0;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts, request a new captcha",
        );
    }

    if !captcha_matches(&stored_captcha, &payload.captcha) {
        return (StatusCode::FORBIDDEN, "Incorrect captcha");
    }

    if let Err((status, message)) = validate_2fa(&state, &payload.two_fa_code) {
        if status == StatusCode::FORBIDDEN {
            // A wrong/malformed 2FA code consumes the captcha immediately:
            // one 2FA attempt per captcha, no retry with the same code.
            state.captcha = None;
            state.captcha_attempts = 0;
            return (StatusCode::FORBIDDEN, INVALID_2FA_CAPTCHA_MESSAGE);
        }
        return (status, message);
    }

    state.last_active = Instant::now();
    // Single-use captcha: force a fresh one for the next keep-alive.
    state.captcha = None;
    state.captcha_attempts = 0;
    (StatusCode::OK, "Status updated")
}

pub async fn status_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let remaining = Duration::from_secs(state.config.timeout_seconds)
        .saturating_sub(state.last_active.elapsed());
    Json(StatusResponse {
        remains_seconds: remaining.as_secs(),
        comment: state
            .config
            .comment
            .clone()
            .unwrap_or_else(|| String::from("^-^")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captcha_comparison_is_case_insensitive() {
        assert!(captcha_matches("AB12CD", "ab12cd"));
        assert!(captcha_matches("ab12cd", "AB12CD"));
        assert!(!captcha_matches("AB12CD", "AB12CE"));
    }
}
