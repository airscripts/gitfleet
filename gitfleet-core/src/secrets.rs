use base64::Engine;
use dryoc::classic::crypto_box::{crypto_box_seal, crypto_box_seal_open, PublicKey, SecretKey};
use dryoc::constants::CRYPTO_BOX_SEALBYTES;

use crate::errors::SecretEncryptionError;

pub fn encrypt_secret(
    value: &str,
    public_key_base64: &str,
) -> Result<String, SecretEncryptionError> {
    let public_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(public_key_base64)
        .map_err(|e| SecretEncryptionError::new(format!("Invalid public key: {e}")))?;

    let public_key: PublicKey = <[u8; 32]>::try_from(public_key_bytes.as_slice())
        .map_err(|e| SecretEncryptionError::new(format!("Invalid public key length: {e}")))?;

    let message = value.as_bytes();

    let mut ciphertext = vec![0u8; message.len() + CRYPTO_BOX_SEALBYTES];
    crypto_box_seal(&mut ciphertext, message, &public_key)
        .map_err(|e| SecretEncryptionError::new(format!("Encryption failed: {e}")))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(ciphertext))
}

pub fn decrypt_secret(
    ciphertext_base64: &str,
    public_key: &PublicKey,
    secret_key: &SecretKey,
) -> Result<String, SecretEncryptionError> {
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_base64)
        .map_err(|e| SecretEncryptionError::new(format!("Invalid ciphertext: {e}")))?;

    if ciphertext.len() < CRYPTO_BOX_SEALBYTES {
        return Err(SecretEncryptionError::new("Ciphertext too short"));
    }

    let mut message = vec![0u8; ciphertext.len() - CRYPTO_BOX_SEALBYTES];
    crypto_box_seal_open(&mut message, &ciphertext, public_key, secret_key)
        .map_err(|e| SecretEncryptionError::new(format!("Decryption failed: {e}")))?;
    String::from_utf8(message)
        .map_err(|e| SecretEncryptionError::new(format!("Invalid UTF-8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_secret_with_invalid_base64() {
        let result = encrypt_secret("test", "not-valid-base64!!!");

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_secret_with_invalid_key_length() {
        let short_key = base64::engine::general_purpose::STANDARD.encode([0u8; 4]);

        let result = encrypt_secret("test", &short_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_secret_round_trip() {
        let (public_key, secret_key) = dryoc::classic::crypto_box::crypto_box_keypair();

        let public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key);
        let encrypted = encrypt_secret("my-secret-value", &public_key_b64).unwrap();

        assert!(!encrypted.is_empty());

        let decrypted = decrypt_secret(&encrypted, &public_key, &secret_key).unwrap();

        assert_eq!(decrypted, "my-secret-value");
    }
}
