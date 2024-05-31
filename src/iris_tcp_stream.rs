use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream, IrisStreamEssentials};

pub struct IrisTcpStream {
    stream: TcpStream,
    buffered_stream: BufReader<TcpStream>,
}

impl IrisTcpStream {
    pub fn new(stream: TcpStream) -> Result<Self, IrisError> {
        let stream_clone = stream
            .try_clone()
            .map_err(|_| IrisError::StreamInitializationError)?;
        Ok(Self {
            stream,
            buffered_stream: BufReader::new(stream_clone),
        })
    }

    pub fn connect(connection_info: String) -> Result<Self, IrisError> {
        let stream = TcpStream::connect(connection_info)
            .map_err(|_| IrisError::StreamInitializationError)?;
        stream
            .set_nodelay(true)
            .map_err(|_| IrisError::StreamInitializationError)?;
        let stream_clone = stream
            .try_clone()
            .map_err(|_| IrisError::StreamInitializationError)?;

        Ok(IrisTcpStream {
            stream,
            buffered_stream: BufReader::new(stream_clone),
        })
    }

    pub fn try_clone(&self) -> Result<Self, std::io::Error> {
        let stream = self.stream.try_clone()?;
        let stream_clone = stream.try_clone()?;
        Ok(Self {
            stream,
            buffered_stream: BufReader::new(stream_clone),
        })
    }
}

impl IrisStreamEssentials for IrisTcpStream {
    fn read_bytes(&mut self, num_bytes: u32) -> Result<Vec<u8>, IrisError> {
        let mut bytes = vec![0; num_bytes.try_into()?];
        self.buffered_stream
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
}

impl IrisStream for IrisTcpStream {}
impl EncryptedIrisStream for IrisTcpStream {}
