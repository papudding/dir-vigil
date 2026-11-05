use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use std::fs::File;
use std::io::{Read, Write};

/// Ensure the encrypted secret file is only readable by its owner on Unix.
fn restrict_permissions(file: &File) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o600);
        file.set_permissions(permissions)?;
    }
    #[cfg(not(unix))]
    {
        let _ = file;
    }
    Ok(())
}

/// 加密数据并将其保存到指定路径
///
/// # 参数
/// - `data`: 需要加密的原始数据
/// - `key`: 加密密钥，必须为32字节长度
/// - `path`: 保存加密后数据的文件路径
///
/// # 返回值
/// - 成功时返回 `Ok(())`
/// - 失败时返回包含错误信息的 `Err`
///
/// # 错误处理
/// - 密钥长度不为32字节时返回错误
/// - 文件创建或写入失败时返回错误
/// - 加密过程失败时返回错误
pub fn encrypt_and_save(
    data: &[u8],
    key: &[u8; 32],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    match cipher.encrypt(&nonce, data) {
        Ok(ciphertext) => {
            let mut file = File::create(path)?;
            restrict_permissions(&file)?;
            file.write_all(nonce.as_slice())?;
            file.write_all(ciphertext.as_slice())?;
            Ok(())
        }
        Err(e) => Err(format!("Encryption failed:{}", e).into()),
    }
}

/// 从指定路径读取并解密数据
///
/// # 参数
/// - `key`: 解密密钥，必须为32字节长度
/// - `path`: 包含加密数据的文件路径
///
/// # 返回值
/// - 成功时返回包含解密数据的 `Vec<u8>`
/// - 失败时返回包含错误信息的 `Err`
///
/// # 错误处理
/// - 文件读取失败时返回错误
/// - 解密过程失败时返回错误
pub fn decrypt_from_file(
    key: &[u8; 32],
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < 12 {
        return Err("Invalid secret file: too short".into());
    }

    let (nonce, ciphertext) = buffer.split_at(12);
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext) {
        Ok(plaintext) => Ok(plaintext),
        Err(e) => Err(format!("Decryption failed: {}", e).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt() {
        let key = b"0123456789abcdef0123456789abcdef";
        let data = b"test data";
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        encrypt_and_save(data, key, path).unwrap();
        let decrypted = decrypt_from_file(key, path).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[cfg(unix)]
    #[test]
    fn test_encrypt_decrypt_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let key = b"0123456789abcdef0123456789abcdef";
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("secret.enc");
        encrypt_and_save(b"test data", key, path.to_str().unwrap()).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_decrypt_too_short_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"short").unwrap();
        let key = b"0123456789abcdef0123456789abcdef";
        assert!(decrypt_from_file(key, temp_file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_invalid_path() {
        let key = b"0123456789abcdef0123456789abcdef";
        let data = b"test data";
        let path = "/invalid/path";

        assert!(encrypt_and_save(data, key, path).is_err());
    }
}
