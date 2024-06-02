mod aes256gcm;
mod xchacha20poly1305;

#[cfg(feature = "clap")]
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::errors::IrisError;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Copy, Default)]
#[cfg_attr(feature = "clap", derive(ValueEnum))]
pub enum CipherType {
    Aes256Gcm,
    #[default]
    XChaCha20Poly1305,
}

pub trait Cipher {
    fn generate_key(&self) -> Vec<u8>;
    /// Returns the encrypted message appended to the generated nonce.
    ///
    /// If there is an error during encryption, returns [`IrisError::CryptoEncryptionError`].
    fn encrypt(&self, message: &[u8]) -> Result<Vec<u8>, IrisError>;
    /// Takes a message of the form nonce + ciphertext and returns the plaintext.
    ///
    /// If there is an error during decryption, returns [`IrisError::CryptoDecryptionError`].
    fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, IrisError>;
}

pub fn get_cipher(cipher_type: CipherType, key: &[u8]) -> Result<Box<dyn Cipher>, IrisError> {
    match cipher_type {
        CipherType::Aes256Gcm => aes256gcm::initiate_cipher(key),
        CipherType::XChaCha20Poly1305 => xchacha20poly1305::initiate_cipher(key),
    }
}
