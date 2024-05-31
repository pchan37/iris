use crate::cipher::Cipher;
use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream, IrisStreamEssentials};
use crate::IrisMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageTracker {
    ReadIrisMessage(IrisMessage),
    WriteIrisMessage(IrisMessage),
    ReadBytes(Vec<u8>),
    WriteBytes(Vec<u8>),
}

pub struct IrisChannelStream {
    pub rx_channel: std::sync::mpsc::Receiver<u8>,
    pub tx_channel: std::sync::mpsc::Sender<u8>,
    pub messages_sent: Vec<MessageTracker>,
}

impl IrisStreamEssentials for IrisChannelStream {
    fn read_bytes(&mut self, num_bytes: u32) -> Result<Vec<u8>, IrisError> {
        let mut bytes = vec![];
        for _ in 0..num_bytes {
            bytes.push(self.rx_channel.recv().unwrap());
        }
        Ok(bytes)
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), IrisError> {
        for byte in bytes.iter() {
            self.tx_channel.send(*byte).unwrap();
        }
        Ok(())
    }
}

impl IrisStream for IrisChannelStream {
    fn read_size_prefixed_message(&mut self) -> Result<Vec<u8>, IrisError> {
        let size_as_bytes = self.read_bytes(u32::BITS / 8)?;
        let size = u32::from_be_bytes(size_as_bytes.try_into().unwrap());

        let message = self.read_bytes(size)?;
        self.messages_sent
            .push(MessageTracker::ReadBytes(message.clone()));
        Ok(message)
    }

    fn read_iris_message(&mut self) -> Result<IrisMessage, IrisError> {
        let serialized_message = self.read_size_prefixed_message()?;
        let message = serde_json::from_slice(&serialized_message)
            .map_err(|_| IrisError::DeserializationError)?;
        self.messages_sent.pop();
        self.messages_sent
            .push(MessageTracker::ReadIrisMessage(message));
        Ok(message)
    }

    fn write_size_prefixed_message(&mut self, bytes: &[u8]) -> Result<(), IrisError> {
        let size_as_bytes = u32::to_be_bytes(bytes.len().try_into()?);
        self.write_bytes(&size_as_bytes)?;
        self.messages_sent
            .push(MessageTracker::WriteBytes(bytes.to_vec()));
        self.write_bytes(bytes)
    }

    fn write_iris_message(&mut self, iris_message: IrisMessage) -> Result<(), IrisError> {
        let serialized_message =
            serde_json::to_vec(&iris_message).map_err(|_| IrisError::SerializationError)?;
        self.write_size_prefixed_message(&serialized_message)?;

        self.messages_sent.pop();
        self.messages_sent
            .push(MessageTracker::WriteIrisMessage(iris_message));

        Ok(())
    }
}

impl EncryptedIrisStream for IrisChannelStream {
    fn read_encrypted_iris_message(
        &mut self,
        cipher: &dyn Cipher,
    ) -> Result<IrisMessage, IrisError> {
        let message = self.read_encrypted_message(cipher)?;
        let iris_message =
            serde_json::from_slice(&message).map_err(|_| IrisError::DeserializationError)?;

        self.messages_sent.pop();
        self.messages_sent
            .push(MessageTracker::ReadIrisMessage(iris_message));

        Ok(iris_message)
    }

    fn write_encrypted_iris_message(
        &mut self,
        cipher: &dyn Cipher,
        iris_message: IrisMessage,
    ) -> Result<(), IrisError> {
        let serialized_message =
            serde_json::to_vec(&iris_message).map_err(|_| IrisError::SerializationError)?;
        self.write_encrypted_message(cipher, &serialized_message)?;

        self.messages_sent.pop();
        self.messages_sent
            .push(MessageTracker::WriteIrisMessage(iris_message));

        Ok(())
    }
}
