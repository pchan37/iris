use std::path::PathBuf;

use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::cipher::CipherType;
use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream};
use crate::iris_tcp_stream::IrisTcpStream;
use crate::room_mapping::RoomIdentifier;
use crate::IrisMessage;

pub fn send(
    server_ip: String,
    server_port: String,
    cipher_type: CipherType,
    files: Vec<PathBuf>,
) -> Result<(), IrisError> {
    let passphrase = "this-is-secret";

    let mut server_connection = IrisTcpStream::connect(format!("{server_ip}:{server_port}"))?;
    server_connection.write_iris_message(IrisMessage::SenderConnecting)?;

    match server_connection.read_iris_message()? {
        IrisMessage::AssignedRoomIdentifier { room_identifier } => {
            tracing::info!("assigned {room_identifier}");
            if matches!(
                server_connection.read_iris_message()?,
                IrisMessage::ReceiverConnected
            ) {
                server_connection.write_iris_message(IrisMessage::SetCipherType { cipher_type })?;
                let key =
                    perform_key_exchange(&mut server_connection, room_identifier, passphrase)?;
                tracing::info!("switching over to encrypted communication");
                Ok(())
            } else {
                Err(IrisError::UnexpectedMessage)
            }
        }
        IrisMessage::ServerError => unreachable!(),
        _ => Err(IrisError::UnexpectedMessage),
    }
}

fn perform_key_exchange(
    server_connection: &mut dyn EncryptedIrisStream,
    room_identifier: RoomIdentifier,
    passphrase: &str,
) -> Result<Vec<u8>, IrisError> {
    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_symmetric(
        &Password::new(passphrase.as_bytes()),
        &Identity::new(format!("iris-{room_identifier}").as_bytes()),
    );
    let receiver_code = server_connection.read_size_prefixed_message()?;
    let key = s1.finish(&receiver_code).map_err(IrisError::SpakeError)?;

    server_connection.write_size_prefixed_message(&outbound_msg)?;

    Ok(key)
}
