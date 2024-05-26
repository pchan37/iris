use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream};
use crate::iris_tcp_stream::IrisTcpStream;
use crate::room_mapping::RoomIdentifier;
use crate::IrisMessage;

#[derive(Debug, Clone, Copy, Default)]
pub enum ConflictingFileMode {
    Overwrite,
    Skip,
    Resume,
    #[default]
    Error,
}

pub fn receive(
    server_ip: String,
    server_port: String,
    passphrase: String,
    conflicting_file_mode: ConflictingFileMode,
) -> Result<(), IrisError> {
    if let Some((room_identifier_str, passphrase)) = passphrase.split_once('-') {
        let room_identifier = room_identifier_str
            .parse::<RoomIdentifier>()
            .map_err(|_| IrisError::InvalidPassphrase)?;
        tracing::debug!("connecting to room #{room_identifier}");

        let mut server_connection = IrisTcpStream::connect(format!("{server_ip}:{server_port}"))?;
        server_connection
            .write_iris_message(IrisMessage::ReceiverConnecting { room_identifier })?;

        match server_connection.read_iris_message()? {
            IrisMessage::SetCipherType { cipher_type } => {
                tracing::debug!("using cipher: {cipher_type:?}");
                let key =
                    perform_key_exchange(&mut server_connection, room_identifier_str, passphrase)?;
                tracing::info!("switching over to encrypted communication");
                Ok(())
            }
            IrisMessage::BadRoomIdentifier => Err(IrisError::InvalidPassphrase),
            _ => Err(IrisError::UnexpectedMessage),
        }
    } else {
        Err(IrisError::InvalidPassphrase)
    }
}

fn perform_key_exchange(
    server_connection: &mut dyn EncryptedIrisStream,
    room_identifier: &str,
    passphrase: &str,
) -> Result<Vec<u8>, IrisError> {
    let (s2, outbound_msg) = Spake2::<Ed25519Group>::start_symmetric(
        &Password::new(passphrase.as_bytes()),
        &Identity::new(format!("iris-{room_identifier}").as_bytes()),
    );
    server_connection.write_size_prefixed_message(&outbound_msg)?;

    let sender_code = server_connection.read_size_prefixed_message()?;
    let key = s2.finish(&sender_code).map_err(IrisError::SpakeError)?;

    Ok(key)
}
