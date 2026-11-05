use clap::Parser;
use std::fmt;
use std::time::Instant;

/// Environment variable that must provide the AES key used to encrypt
/// `user_secret.enc`. Accepts either 32 raw bytes or 64 hex characters.
pub const SECRET_KEY_ENV: &str = "DIR_VIGIL_SECRET_KEY";

#[derive(Parser, Debug, Default, Clone)]
#[command(version, about)]
pub struct Config {
    /// Directory to track for deletion (not required for --print-secret-uri)
    #[arg(short, long, required_unless_present = "print_secret_uri")]
    pub directory: Option<String>,

    /// Timeout in seconds before deletion (24 hours default)
    #[arg(short, long, default_value = "86400")]
    pub timeout_seconds: u64,

    /// Warning in seconds before deletion (6 hours default)
    #[arg(short, long, default_value = "21600")]
    pub warning_seconds: u64,

    /// Checking and alerting interval in seconds (20 minutes default)
    #[arg(short, long, default_value = "1200")]
    pub checking_interval: u64,

    /// Alerting URL (optional; contains a secret token, never logged)
    #[arg(long)]
    pub alert_url: Option<String>,

    /// Alerting channel (bark or ServerChan3)
    #[arg(long, value_parser = ["ServerChan3", "bark"])]
    pub alert_channel: Option<String>,

    /// HTTP listen address. Defaults to loopback; use 0.0.0.0 only behind a trusted proxy.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// HTTP server port
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Allowed CORS origin for local frontend development, e.g. http://localhost:3000.
    /// When unset, no CORS headers are sent (same-origin only).
    #[arg(long)]
    pub cors_origin: Option<String>,

    /// Comment shown on the status page
    #[arg(long)]
    pub comment: Option<String>,

    /// Print the TOTP enrollment QR code/URI and exit (for first-time setup in Docker)
    #[arg(long)]
    pub print_secret_uri: bool,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(directory) = &self.directory {
            writeln!(f, "Directory: {}", directory)?;
        }
        writeln!(f, "Host: {}", self.host)?;
        writeln!(f, "Port: {}", self.port)?;
        writeln!(f, "Timeout seconds: {}", self.timeout_seconds)?;
        writeln!(f, "Warning seconds: {}", self.warning_seconds)?;
        writeln!(f, "Checking interval: {}", self.checking_interval)?;
        if self.alert_url.is_some() {
            writeln!(f, "Alert URL: <configured>")?;
        }
        if let Some(channel) = &self.alert_channel {
            writeln!(f, "Alert channel: {}", channel)?;
        }
        if let Some(origin) = &self.cors_origin {
            writeln!(f, "CORS origin: {}", origin)?;
        }
        Ok(())
    }
}

pub struct AppState {
    pub captcha: Option<String>,
    pub captcha_attempts: u32,
    pub last_active: Instant,
    pub config: Config,
    pub file_encypt_key: [u8; 32],
    pub serect_file_path: String,
}

/// Parse the AES key from an environment value.
///
/// Supported formats:
/// - 32 raw bytes (legacy key format, kept for migrations)
/// - 64 hex characters (recommended, e.g. `openssl rand -hex 32`)
pub fn parse_secret_key(raw: &str) -> Result<[u8; 32], String> {
    let raw = raw.trim();
    if raw.len() == 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(raw.as_bytes());
        return Ok(key);
    }
    if raw.len() == 64 {
        let mut key = [0u8; 32];
        for (index, byte) in key.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&raw[index * 2..index * 2 + 2], 16)
                .map_err(|_| format!("{SECRET_KEY_ENV} contains non-hex characters"))?;
        }
        return Ok(key);
    }
    Err(format!(
        "{SECRET_KEY_ENV} must be 32 raw bytes or 64 hex characters"
    ))
}

/// Read and validate the AES key from the environment.
pub fn secret_key_from_env() -> Result<[u8; 32], String> {
    let raw = std::env::var(SECRET_KEY_ENV).map_err(|_| {
        format!(
            "{SECRET_KEY_ENV} is not set. Generate one with `openssl rand -hex 32` and pass it to the process."
        )
    })?;
    parse_secret_key(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_32_byte_key() {
        let legacy = "0123456789abcdef0123456789abcdef";
        let key = parse_secret_key(legacy).expect("legacy key should parse");
        assert_eq!(&key, legacy.as_bytes());
    }

    #[test]
    fn parses_hex_key() {
        let bytes: [u8; 32] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff, 0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87, 0x98, 0xa9, 0xba, 0xcb,
            0xdc, 0xed, 0xfe, 0x0f,
        ];
        let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(parse_secret_key(&hex).expect("hex key should parse"), bytes);
    }

    #[test]
    fn rejects_invalid_keys() {
        assert!(parse_secret_key("too-short").is_err());
        assert!(parse_secret_key("0123456789abcdef0123456789abcdef").is_ok());
        let invalid_hex = "z".repeat(64);
        assert!(parse_secret_key(&invalid_hex).is_err());
    }
}
