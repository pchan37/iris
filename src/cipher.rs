use crate::errors::IrisError;

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
