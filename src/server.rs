use std::net::TcpListener;

use threadpool::ThreadPool;

use crate::errors::IrisError;
use crate::iris_stream::{EncryptedIrisStream, IrisStream};
use crate::iris_tcp_stream::IrisTcpStream;
use crate::room_mapping::RoomMapping;
use crate::IrisMessage;

pub fn serve(ip_address: String, port: String) -> Result<(), IrisError> {
    let pool = ThreadPool::new(4);

    let listener = TcpListener::bind(format!("{ip_address}:{port}")).unwrap();
    tracing::info!("listening on {ip_address}:{port}");

    let mut room_mapping = RoomMapping::new();
    loop {
        if let Ok((socket, addr)) = listener.accept() {
            // If we cannot convert the socket to a IrisTcpStream, we got a massive
            // problem so the server should return the error and stop.
            let mut socket = IrisTcpStream::new(socket)?;

            if let Ok(message) = socket.read_iris_message() {
                match message {
                    IrisMessage::SenderConnecting => {
                        tracing::debug!("sender #{addr} is connected");
                        if let Ok(mut sender_socket) = socket.try_clone() {
                            let room_identifier = room_mapping.insert_socket(socket);

                            if sender_socket
                                .write_iris_message(IrisMessage::AssignedRoomIdentifier {
                                    room_identifier,
                                })
                                .is_err()
                            {
                                room_mapping.get_and_remove_socket(room_identifier);
                            }
                        } else {
                            tracing::error!("failed to clone the socket");
                            // Ignore the error if sender disconnected, we do not want to bring
                            // down the server as well
                            let _ = socket.write_iris_message(IrisMessage::ServerError);
                        }
                    }
                    IrisMessage::ReceiverConnecting { room_identifier } => {
                        tracing::debug!("receiver #{addr} is connected");
                        let mut receiver_socket = socket;
                        if let Some(mut sender_socket) =
                            room_mapping.get_and_remove_socket(room_identifier)
                        {
                            pool.execute(move || {
                                // Notify sender that receiver is connected
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                sender_socket
                                    .write_iris_message(IrisMessage::ReceiverConnected)
                                    .unwrap();

                                // Forward the SetCipher message
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                sender_socket.forward_message(&mut receiver_socket).unwrap();

                                // Perform the key exchange
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                receiver_socket
                                    .forward_message(sender_socket.as_mut())
                                    .unwrap();
                                sender_socket.forward_message(&mut receiver_socket).unwrap();

                                // Forward the ReadyToReceiveTransfermetadata message
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                receiver_socket
                                    .forward_message(sender_socket.as_mut())
                                    .unwrap();

                                // Forward the total files and size
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                sender_socket.forward_message(&mut receiver_socket).unwrap();

                                // Forward the ReadyToReceiveFiles message
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                receiver_socket
                                    .forward_message(sender_socket.as_mut())
                                    .unwrap();

                                // Relay the files
                                // Fail the entire transaction if sender/receiver is disconnected via unwrap.
                                while sender_socket.forward_message(&mut receiver_socket).is_ok() {
                                    receiver_socket
                                        .forward_message(sender_socket.as_mut())
                                        .unwrap();
                                }

                                tracing::debug!("done relaying");
                            });
                        } else {
                            // Ignore the error if receiver is disconnected, we do not want to bring
                            // down the server as well
                            let _ =
                                receiver_socket.write_iris_message(IrisMessage::BadRoomIdentifier);
                        }
                    }
                    _ => tracing::warn!("detected an unexpected connection"),
                }
            } else {
                tracing::error!("failed to read message");
            }
        }
    }
}
