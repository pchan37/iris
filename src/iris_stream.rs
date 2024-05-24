use core::fmt::Debug;

use crate::cipher::Cipher;
use crate::errors::IrisError;
use crate::IrisMessage;

pub trait IrisStreamEssentials {
    fn read_bytes(&mut self, num_bytes: u32) -> Result<Vec<u8>, IrisError>;
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), IrisError>;
    fn read_ack(&mut self) -> Result<bool, IrisError>;
    fn write_ack(&mut self) -> Result<(), IrisError>;
}

pub trait IrisStream: IrisStreamEssentials {
    fn read_size_prefixed_message(&mut self) -> Result<Vec<u8>, IrisError> {
        let size_as_bytes = self.read_bytes(u32::BITS / 8)?;
        let size = u32::from_be_bytes(size_as_bytes.try_into().unwrap());
        self.write_ack()?;

        let message = self.read_bytes(size)?;
        Ok(message)
    }

    fn read_iris_message(&mut self) -> Result<IrisMessage, IrisError> {
        let serialized_message = self.read_size_prefixed_message()?;
        let message = serde_json::from_slice(&serialized_message)
            .map_err(|_| IrisError::DeserializationError)?;
        Ok(message)
    }

    fn write_size_prefixed_message(&mut self, bytes: &[u8]) -> Result<(), IrisError> {
        let size_as_bytes = u32::to_be_bytes(bytes.len().try_into()?);
        self.write_bytes(&size_as_bytes)?;
        if self.read_ack()? {
            self.write_bytes(bytes)
        } else {
            Err(IrisError::UnexpectedMessage)
        }
    }

    fn write_iris_message(&mut self, iris_message: IrisMessage) -> Result<(), IrisError> {
        let serialized_message =
            serde_json::to_vec(&iris_message).map_err(|_| IrisError::SerializationError)?;
        self.write_size_prefixed_message(&serialized_message)
    }

    fn forward_size_prefixed_message(
        &mut self,
        destination_stream: &mut dyn IrisStream,
    ) -> Result<(), IrisError> {
        let message = self.read_size_prefixed_message()?;
        destination_stream.write_size_prefixed_message(&message)
    }
}

pub trait EncryptedIrisStream: IrisStream {
    fn read_encrypted_message(&mut self, cipher: &dyn Cipher) -> Result<Vec<u8>, IrisError> {
        let nonce_and_ciphertext = self.read_size_prefixed_message()?;
        let message = cipher.decrypt(&nonce_and_ciphertext)?;

        Ok(message)
    }

    fn read_encrypted_iris_message(
        &mut self,
        cipher: &dyn Cipher,
    ) -> Result<IrisMessage, IrisError> {
        let message = self.read_encrypted_message(cipher)?;
        serde_json::from_slice(&message).map_err(|_| IrisError::DeserializationError)
    }

    fn write_encrypted_message(
        &mut self,
        cipher: &dyn Cipher,
        message: &[u8],
    ) -> Result<(), IrisError> {
        let final_message = cipher.encrypt(message)?;
        self.write_size_prefixed_message(&final_message)
    }

    fn write_encrypted_iris_message(
        &mut self,
        cipher: &dyn Cipher,
        iris_message: IrisMessage,
    ) -> Result<(), IrisError> {
        let serialized_message =
            serde_json::to_vec(&iris_message).map_err(|_| IrisError::SerializationError)?;
        self.write_encrypted_message(cipher, &serialized_message)
    }

    fn forward_message(
        &mut self,
        destination_stream: &mut dyn EncryptedIrisStream,
    ) -> Result<(), IrisError> {
        // Following stabilization of feature(trait_upcasting), can just call `self.forward_size_prefixed_message`.
        let message = self.read_size_prefixed_message()?;
        destination_stream.write_size_prefixed_message(&message)
    }
}

impl Debug for dyn EncryptedIrisStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncryptedIrisStream")
    }
}
