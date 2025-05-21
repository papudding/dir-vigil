use clap::Parser;
use std::time::Instant;
use std::fmt;

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

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Directory: {}", self.directory)?;
        writeln!(f, "Port: {}", self.port)?;
        writeln!(f, "Timeout seconds: {}", self.timeout_seconds)?;
        writeln!(f, "Warning seconds: {}", self.warning_seconds)?;
        writeln!(f, "Checking interval: {}", self.checking_interval)?;
        if let Some(url) = &self.alert_url {
            writeln!(f, "Alert URL: {}", url)?;
        }
        if let Some(channel) = &self.alert_channel {
            writeln!(f, "Alert channel: {}", channel)?;
        }
        Ok(())
    }
}

pub struct AppState {
    pub captcha: Option<String>,
    pub last_active: Instant,
    pub config: Config,
    pub file_encypt_key: &'static [u8; 32],
    pub serect_file_path: String,
}
