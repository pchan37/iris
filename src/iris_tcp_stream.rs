use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream, IrisStreamEssentials};
use crate::IrisMessage;

const ACK_SIZE: u32 = 13;

pub struct IrisTcpStream {
    stream: TcpStream,
}

impl IrisTcpStream {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn connect(connection_info: String) -> Result<Self, IrisError> {
        let stream = TcpStream::connect(connection_info)
            .map_err(|_| IrisError::StreamInitializationError)?;
        stream
            .set_nodelay(true)
            .map_err(|_| IrisError::StreamInitializationError)?;
        Ok(IrisTcpStream { stream })
    }

    pub fn try_clone(&self) -> Result<Self, std::io::Error> {
        let stream = self.stream.try_clone()?;
        Ok(Self { stream })
    }
}

impl IrisStreamEssentials for IrisTcpStream {
    fn read_bytes(&mut self, num_bytes: u32) -> Result<Vec<u8>, IrisError> {
        let mut bytes = vec![0; num_bytes.try_into()?];
        BufReader::new(&self.stream)
            .read_exact(&mut bytes)
            .map_err(|_| IrisError::UserConnectionReadError)?;

        Ok(bytes)
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), IrisError> {
        self.stream
            .write_all(bytes)
            .map_err(|_| IrisError::UserConnectionWriteError)?;
        self.stream
            .flush()
            .map_err(|_| IrisError::UserConnectionWriteError)
    }

    fn read_ack(&mut self) -> Result<bool, IrisError> {
        let message = self.read_bytes(ACK_SIZE)?;
        Ok(matches!(
            serde_json::from_slice(&message).map_err(|_| IrisError::DeserializationError)?,
            IrisMessage::Acknowledge
        ))
    }

    fn write_ack(&mut self) -> Result<(), IrisError> {
        let serialized_ack = serde_json::to_vec(&IrisMessage::Acknowledge)
            .map_err(|_| IrisError::SerializationError)?;
        self.write_bytes(&serialized_ack)
    }
}

impl IrisStream for IrisTcpStream {}
impl EncryptedIrisStream for IrisTcpStream {}

#[cfg(test)]
mod tests {
    use crate::iris_tcp_stream::ACK_SIZE;
    use crate::IrisMessage;

    #[test]
    fn test_ack_size_is_correct() {
        let serialized_ack = serde_json::to_vec(&IrisMessage::Acknowledge).unwrap();
        assert_eq!(ACK_SIZE, serialized_ack.len() as u32);
    }
}
