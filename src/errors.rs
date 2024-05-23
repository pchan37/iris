use thiserror::Error;

#[derive(Debug, Error)]
pub enum IrisError {
    /// An error occurred while trying to initialize the cipher for encrypted communication.
    #[error("error in initializing encrypted communication, please reach out to the developer")]
    CryptoInitError,
    /// An error occurred while trying to encrypt a message.
    #[error("error in encrypting communication, please reach out to the developer")]
    CryptoEncryptionError,
    /// An error occurred while trying to decrypt a message.
    #[error("error in decrypting communication, please reach out to the developer")]
    CryptoDecryptionError,
}
