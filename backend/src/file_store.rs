use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use std::fs::File;
use std::io::{Read, Write};
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

    #[test]
    fn test_invalid_path() {
        let key = b"0123456789abcdef0123456789abcdef";
        let data = b"test data";
        let path = "/invalid/path";

        assert!(encrypt_and_save(data, key, path).is_err());
    }
}
