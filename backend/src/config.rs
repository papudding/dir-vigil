use clap::Parser;
use std::time::Instant;

#[derive(Parser, Debug, Default, Clone)]
#[command(version, about)]
pub struct Config {
    /// Directory to track for deletion
    #[arg(short, long)]
    pub directory: String,

    /// Timeout in seconds before deletion (24 hours default)
    #[arg(short, long, default_value = "86400")]
    pub timeout_seconds: u64,

    /// warning in seconds before deletion (6 hours default)
    #[arg(short, long, default_value = "21600")]
    pub warning_seconds: u64,

    /// checking and alerting interval of warning_seconds (20 minutes default)
    #[arg(short, long, default_value = "1200")]
    pub checking_interval: u64,

    /// alerting url
    #[arg(long)]
    pub alert_url: Option<String>,

    /// alerting Channel
    #[arg(long, value_parser = ["ServerChan3", "bark"])]
    pub alert_channel: Option<String>,

    /// HTTP server port
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// comment
    #[arg(long)]
    pub comment: Option<String>,
}

pub struct AppState {
    pub captcha: Option<String>,
    pub last_active: Instant,
    pub config: Config,
    pub file_encypt_key: &'static [u8; 32],
    pub serect_file_path: String,
}
