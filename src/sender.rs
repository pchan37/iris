use std::path::{Path, PathBuf};

use jwalk::WalkDirGeneric;
use spake2::{Ed25519Group, Identity, Password, Spake2};

use crate::cipher::{get_cipher, CipherType};
use crate::errors::IrisError;
use crate::files::{FileMetadata, FileType};
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

                send_transfer_metadata(&mut server_connection, cipher_type, &key, files)
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

fn send_transfer_metadata(
    server_connection: &mut dyn EncryptedIrisStream,
    cipher_type: CipherType,
    encryption_key: &[u8],
    files: Vec<PathBuf>,
) -> Result<(), IrisError> {
    let cipher = get_cipher(cipher_type, encryption_key)?;

    let (complete_file_list, total_size) = get_complete_file_list_and_total_size(files)?;
    tracing::info!(
        "going to send {total_size} bytes distributed among {} files",
        complete_file_list.len()
    );

    let iris_message = server_connection.read_encrypted_iris_message(&*cipher)?;
    if !matches!(iris_message, IrisMessage::ReadyToReceiveMetadata) {
        return Err(IrisError::UnexpectedMessage);
    }
    server_connection.write_encrypted_iris_message(
        &*cipher,
        IrisMessage::TransferMetadata {
            total_files: complete_file_list.len(),
            total_bytes: total_size,
        },
    )?;

    let iris_message = server_connection.read_encrypted_iris_message(&*cipher)?;
    if !matches!(iris_message, IrisMessage::ReadyToReceiveFiles) {
        return Err(IrisError::UnexpectedMessage);
    }
    tracing::info!("sending files");

    Ok(())
}

fn get_complete_file_list_and_total_size(
    files: Vec<PathBuf>,
) -> Result<(Vec<(PathBuf, FileMetadata)>, u64), IrisError> {
    let mut complete_file_list = Vec::new();
    let mut total_bytes_to_send = 0;

    for file_path in files.iter() {
        tracing::debug!("walking into {}", file_path.display());
        let canonicalized_file_path = canonicalize_path(file_path)?;

        for dir_entry_result in walk_path(file_path) {
            match dir_entry_result {
                Ok(dir_entry) => {
                    let mut dest_path = canonicalized_file_path.clone();
                    if let Ok(file_path_without_prefix) = dir_entry.path().strip_prefix(file_path) {
                        if file_path_without_prefix.file_name().is_some() {
                            dest_path.push(file_path_without_prefix);
                        }
                    }

                    let file_metadata = match dir_entry.client_state {
                        Some(Ok(file_size)) => {
                            FileMetadata::new(dest_path, FileType::File, file_size)
                        }
                        Some(Err(_)) => {
                            return Err(IrisError::PermissionsUserIOError(
                                file_path.display().to_string(),
                            ))
                        }
                        None => FileMetadata::new(dest_path, FileType::Directory, 0),
                    };

                    total_bytes_to_send += file_metadata.get_size();
                    complete_file_list.push((dir_entry.path(), file_metadata));
                }
                Err(e) => {
                    return Err(IrisError::PermissionsUserIOError(
                        e.path().unwrap_or(Path::new("")).display().to_string(),
                    ))
                }
            }
        }
    }

    Ok((complete_file_list, total_bytes_to_send))
}

fn walk_path(root: &PathBuf) -> WalkDirGeneric<((), Option<Result<u64, jwalk::Error>>)> {
    WalkDirGeneric::<((), Option<Result<u64, jwalk::Error>>)>::new(root)
        .skip_hidden(false)
        .process_read_dir(|_, _, _, dir_entry_results| {
            dir_entry_results.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if !dir_entry.file_type.is_dir() {
                        dir_entry.client_state = Some(dir_entry.metadata().map(|m| m.len()));
                    }
                }
            })
        })
}

fn canonicalize_path(file_path: &Path) -> Result<PathBuf, IrisError> {
    Ok(file_path
        .canonicalize()
        .map_err(|_| IrisError::PermissionsUserIOError(file_path.display().to_string()))?
        .file_name()
        .ok_or(IrisError::PermissionsUserIOError(
            file_path.display().to_string(),
        ))?
        .into())
}
