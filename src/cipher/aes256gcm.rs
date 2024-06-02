use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, KeyInit};
use rand::rngs::OsRng;

use crate::errors::IrisError;

use super::Cipher;

const NONCE_SIZE: usize = 12;

pub struct Aes256GcmCipher {
    cipher: Aes256Gcm,
}

pub fn initiate_cipher(key: &[u8]) -> Result<Box<dyn Cipher>, IrisError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| IrisError::CryptoInitError)?;
    Ok(Box::new(Aes256GcmCipher { cipher }))
}

impl Cipher for Aes256GcmCipher {
    fn generate_key(&self) -> Vec<u8> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        key.to_vec()
    }

    fn encrypt(&self, message: &[u8]) -> Result<Vec<u8>, IrisError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
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
