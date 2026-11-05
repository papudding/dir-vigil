use crate::config::{AppState, Config};
use log::{debug, error, info, warn};
use qrcode::QrCode;
use std::io::{self, Write};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub async fn monitor_task(state: Arc<Mutex<AppState>>) {
    let check_interval = Duration::from_secs(1);
    info!("Monitoring started...");

    loop {
        tokio::time::sleep(check_interval).await;

        let state = state.lock().await;
        let elapsed = state.last_active.elapsed();
        let timeout = Duration::from_secs(state.config.timeout_seconds);
        print!(
            "Remaining time: {} seconds \r",
            timeout.saturating_sub(elapsed).as_secs()
        );
        io::stdout().flush().unwrap(); // 刷新输出缓冲区
        if elapsed >= timeout {
            let directory = state.config.directory.clone().unwrap_or_default();
            drop(state);
            cleanup_directory(&directory);
            std::process::exit(0);
        }
    }
}

/// Trigger alert if remaining time is less than warning_seconds
pub async fn check_remain_time(state: Arc<Mutex<AppState>>, config: Config) {
    info!("check remain time started...");
    loop {
        tokio::time::sleep(Duration::from_secs(config.checking_interval)).await;
        let state = state.lock().await;
        let elapsed = state.last_active.elapsed();
        let timeout = Duration::from_secs(config.timeout_seconds);
        let remains_time = timeout.saturating_sub(elapsed);
        let warning_seconds = Duration::from_secs(config.warning_seconds);
        info!(
            "Remaining time: {} seconds, warning threshold: {} seconds",
            remains_time.as_secs(),
            warning_seconds.as_secs()
        );
        if remains_time < warning_seconds {
            info!(
                "Warning! Remaining time: {} is below warning_seconds: {}",
                remains_time.as_secs(),
                config.warning_seconds
            );
            if let Err(e) = send_alert_request(&config, remains_time).await {
                error!("Failed to send alert request: {}", e);
            }
        }
    }
}

pub fn build_alert_body(alert_channel: &str, remains_time: Duration) -> String {
    let remain_hours = remains_time.as_secs() / 3600;
    let remain_minutes = (remains_time.as_secs() % 3600) / 60;
    match alert_channel {
        "ServerChan3" => format!(
            "{{\
              \"title\" : \"dir-vigil\",\
            \"desp\" : \"Directory will be deleted after {} hours and {} minutes!\"\
        }}",
            remain_hours, remain_minutes
        ),
        "bark" => format!(
            "{{\
            \"title\" : \"dir-vigil\",\
            \"body\": \"Directory will be deleted after {} hours and {} minutes!\"\
        }}",
            remain_hours, remain_minutes
        ),
        _ => format!(
            "Directory will be deleted after {} hours and {} minutes!",
            remain_hours, remain_minutes
        ),
    }
}

pub async fn send_alert_request(
    config: &Config,
    remains_time: Duration,
) -> Result<(), reqwest::Error> {
    let (Some(alert_channel), Some(alert_url)) =
        (config.alert_channel.as_deref(), config.alert_url.as_deref())
    else {
        warn!("No alert URL or channel configured, skipping alert request");
        return Ok(());
    };

    // Never log the alert URL: it often contains a private token.
    info!("Sending alert request via {alert_channel}");
    let body = build_alert_body(alert_channel, remains_time);

    let client = reqwest::Client::new();
    let response = client
        .post(alert_url)
        .body(body)
        .header("Content-Type", "application/json; charset=utf-8")
        .send()
        .await?;

    if !response.status().is_success() {
        error!("Alert request failed with status: {}", response.status());
    } else {
        debug!("Alert request completed with status: {}", response.status());
    }

    Ok(())
}

pub fn print_qrcode(data: &str) {
    let qr = QrCode::new(data).unwrap();
    let string = qr
        .render()
        .quiet_zone(true)
        .dark_color("█")
        .module_dimensions(2, 1)
        .build();

    println!("{}", string);
}

pub fn cleanup_directory(dir: &str) {
    let path = Path::new(dir);

    if path.exists() {
        if let Err(e) = std::fs::remove_dir_all(path) {
            error!("Failed to delete directory: {}", e);
        } else {
            info!("Successfully deleted directory: {}", dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_alert_request_without_url() {
        let config = Config {
            alert_url: None,
            ..Default::default()
        };
        assert!(send_alert_request(&config, Duration::from_secs(1))
            .await
            .is_ok());
    }

    #[test]
    fn test_build_alert_body_server_chan3() {
        let body = build_alert_body("ServerChan3", Duration::from_secs(3_660));
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["title"], "dir-vigil");
        assert_eq!(
            json["desp"],
            "Directory will be deleted after 1 hours and 1 minutes!"
        );
    }

    #[test]
    fn test_build_alert_body_bark() {
        let body = build_alert_body("bark", Duration::from_secs(7_200));
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["title"], "dir-vigil");
        assert_eq!(
            json["body"],
            "Directory will be deleted after 2 hours and 0 minutes!"
        );
    }

    #[test]
    fn test_build_alert_body_unknown_channel() {
        let body = build_alert_body("unknown", Duration::from_secs(60));
        assert_eq!(
            body,
            "Directory will be deleted after 0 hours and 1 minutes!"
        );
    }
}
