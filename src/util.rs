use crate::config::AppState;
use qrcode::QrCode;
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use std::io::{self, Write};

pub async fn monitor_task(state: Arc<Mutex<AppState>>) {
    let check_interval = Duration::from_secs(1);
    println!("Monitoring started...");

    loop {
        tokio::time::sleep(check_interval).await;

        let state = state.lock().await;
        let elapsed = state.last_active.elapsed();
        let timeout = Duration::from_secs(state.config.timeout_seconds);
        print!("Remaining time: {} seconds \r", timeout.as_secs() - elapsed.as_secs());
        io::stdout().flush().unwrap(); // 刷新输出缓冲区
        if elapsed >= timeout {
            cleanup_directory(&state.config.directory);
            std::process::exit(0);
        }
    }
}

/// Trigger alert if remaining time is less than warning_seconds
pub async fn check_remain_time(state: Arc<Mutex<AppState>>) {
    let check_interval = Duration::from_secs(60 * 30); // 每30分钟检查一次
    println!("check remain time started...");
    loop {
        tokio::time::sleep(check_interval).await;
        let state = state.lock().await;
        let elapsed = state.last_active.elapsed();
        let timeout = Duration::from_secs(state.config.timeout_seconds);
        let remains_time = timeout - elapsed;
        let warning_seconds = Duration::from_secs(state.config.warning_seconds); 
        if remains_time < warning_seconds {
            alert();
        }
    }
}

pub fn alert() {
    // todo 
    println!("Alert: Directory has not been accessed in 12 hours!");
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
            eprintln!("Failed to delete directory: {}", e);
        } else {
            println!("Successfully deleted directory: {}", dir);
        }
    }
}
