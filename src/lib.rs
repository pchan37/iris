mod cipher;
mod constants;
mod default_wordlist;
mod errors;
mod files;
#[doc(hidden)]
pub mod iris_channel_stream;
pub mod iris_stream;
mod iris_tcp_stream;
mod passphrase;
mod receiver;
mod room_mapping;
mod sender;
mod server;

use serde::{Deserialize, Serialize};

pub use crate::cipher::CipherType;
pub use crate::default_wordlist::WORDLIST;
pub use crate::passphrase::{
    get_passphrase_from_str_wordlist, get_passphrase_from_string_wordlist,
};
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
    DirectoryCreated,
    FileSkipped,
    FileStartAtPos {
        start_pos: u64,
    },
    ChunkReceived {
        is_last: bool,
    },
    UnexpectedMessage,
    ServerError,
    BadRoomIdentifier,
}
