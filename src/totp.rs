use otpauth::TOTP;
use rand::Rng;

pub fn generate_totp_secret() -> Vec<u8> {
    let mut rng = rand::rng();
    let mut bytes = [0u8; 20];
    rng.fill(&mut bytes);
    println!("generate_totp_secret::bytes: {:?}", bytes);
    Vec::from(bytes)
}

pub fn generate_totp_uri(secret: &[u8], label: &str, issuer: &str) -> String {
    let totp = TOTP::from_bytes(&secret);
    totp.to_uri(label, issuer)
}

pub fn verify_totp_code(secret: &[u8], code: u32, period: u64, timestamp: u64) -> bool {
    let totp = TOTP::from_bytes(&secret);
    totp.verify(code, period, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_totp_secret() {
        let secret = generate_totp_secret();
        println!("test_generate_totp_secret: {:?}", secret);
        assert_eq!(secret.len(), 32);
    }

    #[test]
    fn test_generate_totp_uri() {
        let secret = generate_totp_secret();
        let uri = generate_totp_uri(&secret, "test", "issuer");
        println!("test_g_t_u-secret: {:?}", secret);
        println!("test_g_t_u-uri: {}", uri);
        assert!(uri.contains("otpauth://totp/test?"));
        assert!(uri.contains("issuer=issuer"));
    }
}
