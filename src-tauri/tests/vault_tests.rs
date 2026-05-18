#[cfg(test)]
mod tests {
    use genesis_soc_auth::security::vault::Vault;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0u8; 32]; // Test key, not used in production
        let plaintext = b"Genesis SOC Auth secret seed - 1234567890";

        let encrypted = Vault::encrypt(&key, plaintext)
            .expect("Encryption should succeed");

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len()); // Nonce + tag overhead

        let decrypted = Vault::decrypt(&key, &encrypted)
            .expect("Decryption should succeed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let key = [0u8; 32];
        let plaintext = b"same plaintext";

        let encrypted1 = Vault::encrypt(&key, plaintext).unwrap();
        let encrypted2 = Vault::encrypt(&key, plaintext).unwrap();

        // Each encryption should produce different ciphertext due to random nonce
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let plaintext = b"test data";

        let encrypted = Vault::encrypt(&key1, plaintext).unwrap();
        let result = Vault::decrypt(&key2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = [0u8; 32];
        let plaintext = b"test data";

        let mut encrypted = Vault::encrypt(&key, plaintext).unwrap();
        // Tamper with a byte in the ciphertext portion
        if let Some(byte) = encrypted.get_mut(14) {
            *byte ^= 0xFF;
        }

        let result = Vault::decrypt(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let key = [0u8; 32];
        let encrypted = Vault::encrypt(&key, b"").unwrap();
        let decrypted = Vault::decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, b"");
    }
}
