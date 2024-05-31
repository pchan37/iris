mod cipher;
mod constants;
mod errors;
mod files;
#[doc(hidden)]
pub mod iris_channel_stream;
pub mod iris_stream;
mod iris_tcp_stream;
mod receiver;
mod room_mapping;
mod sender;
mod server;

use serde::{Deserialize, Serialize};

pub use crate::cipher::CipherType;
pub use crate::receiver::{receive, simple_receive, ConflictingFileMode};
use crate::room_mapping::RoomIdentifier;
pub use crate::sender::{send, simple_send};
pub use crate::server::serve;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Copy)]
pub enum IrisMessage {
    Acknowledge,
    SenderConnecting,
    AssignedRoomIdentifier {
        room_identifier: RoomIdentifier,
    },
    ReceiverConnecting {
        room_identifier: RoomIdentifier,
    },
    ReceiverConnected,
    SetCipherType {
        cipher_type: CipherType,
    },
    ReadyToReceiveMetadata,
    TransferMetadata {
        total_files: usize,
        total_bytes: u64,
    },
    ReadyToReceiveFiles,
    UnexpectedMessage,
    ServerError,
    BadRoomIdentifier,
}
