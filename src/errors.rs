use std::num::TryFromIntError;

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
    /// An error occurred while trying to marshal data into json format.
    #[error("error in marshalling data into json format, please reach out to the developer")]
    SerializationError,
    /// An error occurred while trying to unmarshal data from json format.
    #[error("error in unmarshalling data from json format, please reach out to the developer")]
    DeserializationError,
    /// During communication, received an unexpected message signaling either a bug or malicious
    /// activity.
    #[error("received an unexpected message, please reach out to the developer")]
    UnexpectedMessage,
    /// This should only come up when trying to typecast a u32 to a usize smaller than 32 bits which
    /// should not happen on major platforms. Fixing this is low priority at the moment.
    #[error("your platform is currently not supported")]
    U32TypecastError(#[from] TryFromIntError),
    /// The bad connection info was given or unable to set_nodelay to true.
    #[error("unable to make a connection, please confirm the server ip address and port")]
    StreamInitializationError,
    /// Reading from the stream failed.
    #[error("unable to receive data over the connection, please ensure that you are still connected to the other party")]
    UserConnectionReadError,
    /// Writing to the stream failed.
    #[error("unable to send data over the connection, please ensure that you are still connected to the other party")]
    UserConnectionWriteError,
}
