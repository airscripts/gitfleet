use base64::Engine;
use dryoc::classic::crypto_box::crypto_box_keypair;
use gitfleet_core::secrets::{decrypt_secret, encrypt_secret};

#[test]
fn test_encrypt_secret_invalid_base64() {
    let result = encrypt_secret("test", "not-valid-base64!!!");

    assert!(result.is_err());
}

#[test]
fn test_encrypt_secret_invalid_key_length() {
    let short_key = base64::engine::general_purpose::STANDARD.encode([0u8; 4]);

    let result = encrypt_secret("test", &short_key);

    assert!(result.is_err());
}

#[test]
fn test_encrypt_decrypt_round_trip() {
    let (public_key, secret_key) = crypto_box_keypair();

    let public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key);
    let encrypted = encrypt_secret("my-secret-value", &public_key_b64).unwrap();

    assert!(!encrypted.is_empty());

    let decrypted = decrypt_secret(&encrypted, &public_key, &secret_key).unwrap();

    assert_eq!(decrypted, "my-secret-value");
}

#[test]
fn test_encrypt_decrypt_empty_string() {
    let (public_key, secret_key) = crypto_box_keypair();

    let public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key);
    let encrypted = encrypt_secret("", &public_key_b64).unwrap();

    let decrypted = decrypt_secret(&encrypted, &public_key, &secret_key).unwrap();

    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_decrypt_unicode() {
    let (public_key, secret_key) = crypto_box_keypair();

    let public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key);
    let encrypted = encrypt_secret("日本語テスト 🦀", &public_key_b64).unwrap();

    let decrypted = decrypt_secret(&encrypted, &public_key, &secret_key).unwrap();

    assert_eq!(decrypted, "日本語テスト 🦀");
}

#[test]
fn test_decrypt_secret_invalid_ciphertext() {
    let (public_key, _secret_key) = crypto_box_keypair();

    let result = decrypt_secret("not-valid-base64!!!", &public_key, &_secret_key);

    assert!(result.is_err());
}

#[test]
fn test_decrypt_secret_truncated_ciphertext() {
    let (public_key, _secret_key) = crypto_box_keypair();

    let short = base64::engine::general_purpose::STANDARD.encode([0u8; 4]);
    let result = decrypt_secret(&short, &public_key, &_secret_key);

    assert!(result.is_err());
}

#[test]
fn test_different_secrets_produce_different_ciphertexts() {
    let (public_key, _) = crypto_box_keypair();

    let public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key);
    let encrypted_a = encrypt_secret("secret-a", &public_key_b64).unwrap();

    let encrypted_b = encrypt_secret("secret-b", &public_key_b64).unwrap();

    assert_ne!(encrypted_a, encrypted_b);
}
