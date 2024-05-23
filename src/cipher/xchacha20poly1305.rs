use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305};

use crate::errors::IrisError;

use super::Cipher;

const NONCE_SIZE: usize = 24;

pub struct XChaChaPoly1305Cipher {
    cipher: XChaCha20Poly1305,
}

pub fn initiate_cipher(key: &[u8]) -> Result<Box<dyn Cipher>, IrisError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| IrisError::CryptoInitError)?;
    Ok(Box::new(XChaChaPoly1305Cipher { cipher }))
}

impl Cipher for XChaChaPoly1305Cipher {
    fn generate_key(&self) -> Vec<u8> {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        key.to_vec()
    }

    fn encrypt(&self, message: &[u8]) -> Result<Vec<u8>, IrisError> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, message)
            .map_err(|_| IrisError::CryptoEncryptionError)?;

        let complete_message = [nonce.as_slice(), &ciphertext].concat();
        Ok(complete_message)
    }

    fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, IrisError> {
        let (nonce, ciphertext) = (&message[..NONCE_SIZE], &message[NONCE_SIZE..]);

        self.cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|_| IrisError::CryptoDecryptionError)
    }
}
