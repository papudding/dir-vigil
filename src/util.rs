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
