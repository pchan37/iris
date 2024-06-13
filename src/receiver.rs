use std::path::Path;

#[cfg(feature = "clap")]
use clap::ValueEnum;
use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::cipher::{get_cipher, Cipher, CipherType};
use crate::constants::CHUNK_SIZE;
use crate::errors::IrisError;
use crate::files::{File, FileMetadata, FileType};
use crate::iris_stream::{EncryptedIrisStream, IrisStream};
use crate::iris_tcp_stream::IrisTcpStream;
use crate::progress::{ReceiverProgressCommunication, ReceiverProgressMessage, WorkerMessage};
use crate::room_mapping::RoomIdentifier;
use crate::IrisMessage;

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "clap", derive(ValueEnum))]
pub enum ConflictingFileMode {
    Overwrite,
    Skip,
    Resume,
    #[default]
    Error,
}

pub fn simple_receive(
    server_ip: String,
    server_port: String,
    room_identifier_str: &str,
    passphrase: &str,
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    let room_identifier = room_identifier_str
        .parse::<RoomIdentifier>()
        .map_err(|_| IrisError::InvalidPassphrase)?;
    tracing::debug!("connecting to room #{room_identifier}");

    let mut server_connection = IrisTcpStream::connect(format!("{server_ip}:{server_port}"))?;
    server_connection.write_iris_message(IrisMessage::ReceiverConnecting { room_identifier })?;

    receive(
        &mut server_connection,
        room_identifier,
        passphrase,
        conflicting_file_mode,
        progress_communication,
    )
}

pub fn receive(
    server_connection: &mut dyn EncryptedIrisStream,
    room_identifier: RoomIdentifier,
    passphrase: &str,
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    match server_connection.read_iris_message()? {
        IrisMessage::SetCipherType { cipher_type } => {
            tracing::debug!("using cipher: {cipher_type:?}");
            let key = perform_key_exchange(server_connection, room_identifier, passphrase)?;
            progress_communication.write(ReceiverProgressMessage::SetCipher { cipher_type })?;
            tracing::info!("switching over to encrypted communication");

            receive_transfer_metadata(
                server_connection,
                cipher_type,
                &key,
                progress_communication,
            )?;
            receive_files(
                server_connection,
                cipher_type,
                &key,
                conflicting_file_mode,
                progress_communication,
            )
        }
        IrisMessage::BadRoomIdentifier => Err(IrisError::InvalidPassphrase),
        _ => Err(IrisError::UnexpectedMessage),
    }
}

fn perform_key_exchange(
    server_connection: &mut dyn EncryptedIrisStream,
    room_identifier: RoomIdentifier,
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

fn receive_transfer_metadata(
    server_connection: &mut dyn EncryptedIrisStream,
    cipher_type: CipherType,
    decryption_key: &[u8],
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    let cipher = get_cipher(cipher_type, decryption_key)?;

    server_connection
        .write_encrypted_iris_message(&*cipher, IrisMessage::ReadyToReceiveMetadata)?;
    match server_connection.read_encrypted_iris_message(&*cipher)? {
        IrisMessage::TransferMetadata {
            total_files,
            total_bytes,
        } => {
            tracing::info!(
                "going to receive {total_bytes} bytes distributed among {total_files} files"
            );
            progress_communication.write(ReceiverProgressMessage::TransferMetadata {
                total_files,
                total_bytes,
            })?;
            server_connection
                .write_encrypted_iris_message(&*cipher, IrisMessage::ReadyToReceiveFiles)
        }
        _ => Err(IrisError::UnexpectedMessage),
    }
}

fn receive_files(
    server_connection: &mut dyn EncryptedIrisStream,
    cipher_type: CipherType,
    decryption_key: &[u8],
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    let cipher = get_cipher(cipher_type, decryption_key)?;
    while let Ok(raw_file_metadata) = server_connection.read_encrypted_message(&*cipher) {
        let file_metadata = serde_json::from_slice::<FileMetadata>(&raw_file_metadata)
            .map_err(|_| IrisError::DeserializationError)?;
        tracing::debug!("received the following metadata: {file_metadata:?}");
        progress_communication.write(ReceiverProgressMessage::FileMetadata {
            filename: file_metadata.get_filename().to_path_buf(),
            file_size: file_metadata.get_size(),
        })?;

        match file_metadata.get_file_type() {
            FileType::Directory => process_directory(
                server_connection,
                &*cipher,
                file_metadata,
                conflicting_file_mode,
                progress_communication,
            )?,
            FileType::File => process_file(
                server_connection,
                &*cipher,
                file_metadata,
                conflicting_file_mode,
                progress_communication,
            )?,
        }

        if matches!(progress_communication.read()?, Some(WorkerMessage::Cancel)) {
            tracing::debug!("exiting as user cancel");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn process_directory(
    server_connection: &mut dyn EncryptedIrisStream,
    cipher: &dyn Cipher,
    file_metadata: FileMetadata,
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    match conflicting_file_mode {
        ConflictingFileMode::Overwrite => {
            let _ = std::fs::remove_dir_all(file_metadata.get_filename());
            std::fs::create_dir(file_metadata.get_filename()).map_err(|_| {
                IrisError::PermissionsUserIOError(
                    file_metadata.get_filename().display().to_string(),
                )
            })?;
        }
        ConflictingFileMode::Skip | ConflictingFileMode::Resume => {
            if std::fs::create_dir(file_metadata.get_filename()).is_err() {
                progress_communication.write(ReceiverProgressMessage::FileSkipped)?;
                return server_connection
                    .write_encrypted_iris_message(cipher, IrisMessage::FileSkipped);
            }
        }
        ConflictingFileMode::Error => {
            std::fs::create_dir(file_metadata.get_filename()).map_err(|_| {
                IrisError::AlreadyExistsUserIOError(
                    file_metadata.get_filename().display().to_string(),
                )
            })?;
        }
    }

    tracing::debug!("created directory");
    progress_communication.write(ReceiverProgressMessage::DirectoryCreated)?;
    server_connection.write_encrypted_iris_message(cipher, IrisMessage::DirectoryCreated)?;
    Ok(())
}

fn process_file(
    server_connection: &mut dyn EncryptedIrisStream,
    cipher: &dyn Cipher,
    file_metadata: FileMetadata,
    conflicting_file_mode: ConflictingFileMode,
    progress_communication: &ReceiverProgressCommunication,
) -> Result<(), IrisError> {
    let filename = file_metadata.get_filename();

    let (mut file, file_start_pos) = match get_file_and_start_pos(filename, conflicting_file_mode)?
    {
        Some((file, start_pos)) => {
            if start_pos == file_metadata.get_size() {
                tracing::debug!("entire file already transferred, skipping");
                progress_communication.write(ReceiverProgressMessage::FileSkipped)?;
                return server_connection
                    .write_encrypted_iris_message(cipher, IrisMessage::FileSkipped);
            } else {
                progress_communication
                    .write(ReceiverProgressMessage::ChunkReceived { size: start_pos })?;
                server_connection.write_encrypted_iris_message(
                    cipher,
                    IrisMessage::FileStartAtPos { start_pos },
                )?;
                (file, start_pos)
            }
        }
        None => {
            progress_communication.write(ReceiverProgressMessage::FileSkipped)?;
            return server_connection
                .write_encrypted_iris_message(cipher, IrisMessage::FileSkipped);
        }
    };

    let mut bytes_left_to_read = file_metadata.get_size() - file_start_pos;
    while bytes_left_to_read > 0 {
        tracing::debug!("still have {bytes_left_to_read} bytes");

        let file_chunk = server_connection.read_encrypted_message(cipher)?;
        tracing::debug!("got chunk of size: {} bytes", file_chunk.len());
        file.write_chunk(&file_chunk)?;
        tracing::debug!("wrote chunk");

        progress_communication.write(ReceiverProgressMessage::ChunkReceived {
            size: CHUNK_SIZE.min(bytes_left_to_read),
        })?;
        bytes_left_to_read = bytes_left_to_read.saturating_sub(CHUNK_SIZE);
        server_connection.write_encrypted_iris_message(
            cipher,
            IrisMessage::ChunkReceived {
                is_last: bytes_left_to_read == 0,
            },
        )?;

        if matches!(progress_communication.read()?, Some(WorkerMessage::Cancel)) {
            tracing::debug!("exiting as user cancel");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn get_file_and_start_pos(
    filename: &Path,
    conflicting_file_mode: ConflictingFileMode,
) -> Result<Option<(File, u64)>, IrisError> {
    match conflicting_file_mode {
        ConflictingFileMode::Overwrite => {
            let file = File::open_in_overwrite(filename.to_path_buf())?;
            Ok(Some((file, 0)))
        }
        ConflictingFileMode::Skip => match File::open_new_in_append(filename.to_path_buf()) {
            Ok(file) => Ok(Some((file, 0))),
            Err(_) => Ok(None),
        },
        ConflictingFileMode::Resume => {
            let file = File::open_in_append(filename.to_path_buf())?;
            let file_size = file
                .get_size()
                .map_err(|_| IrisError::PermissionsUserIOError(filename.display().to_string()))?;
            Ok(Some((file, file_size)))
        }
        ConflictingFileMode::Error => {
            let file = File::open_new_in_append(filename.to_path_buf())?;
            Ok(Some((file, 0)))
        }
    }
}
