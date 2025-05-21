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
            timeout.as_secs() - elapsed.as_secs()
        );
        io::stdout().flush().unwrap(); // 刷新输出缓冲区
        if elapsed >= timeout {
            cleanup_directory(&state.config.directory);
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
        let remains_time = timeout - elapsed;
        let warning_seconds = Duration::from_secs(config.warning_seconds);
        info!(
            "Remaining time: {} seconds, warning threshold: {} seconds",
            remains_time.as_secs(),
            warning_seconds.as_secs()
        );
        if remains_time < warning_seconds {
            info!("Warning! Remaining time: {} is below warning_seconds: {}", remains_time.as_secs(), config.warning_seconds);
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
    if config.alert_url.is_none() || config.alert_channel.is_none() {
        warn!("No alert URL or channel configured, skipping alert request");
        return Ok(());
    }

    let alert_channel = config.alert_channel.as_ref().map(|s| s.as_str());
    let alert_url = config.alert_url.as_ref().unwrap();

    info!(
        "Sending alert request to {} via {}",
        alert_url, alert_channel.unwrap()
    );
    let body = build_alert_body(alert_channel.unwrap(), remains_time);

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
        debug!("Alert request success : {:?}", response);
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

    #[tokio::test]
    async fn test_send_alert_request_success_with_server_chan3() {
        let config = Config {
            alert_url: Some(String::from("<your_ServerChain3_url>")),
            alert_channel: Some(String::from("ServerChan3")),
            ..Default::default()
        };

        assert!(send_alert_request(&config, Duration::from_secs(1))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_send_alert_request_success_bark() {
        let config = Config {
            alert_url: Some(String::from("<your_bark_url>")),
            alert_channel: Some(String::from("bark")),
            ..Default::default()
        };

        assert!(send_alert_request(&config, Duration::from_secs(1))
            .await
            .is_ok());
    }
}
