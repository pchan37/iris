mod cipher;
mod constants;
mod errors;
mod files;
mod iris_stream;
mod iris_tcp_stream;
mod receiver;
mod room_mapping;
mod sender;
mod server;

use serde::{Deserialize, Serialize};

pub use crate::cipher::CipherType;
pub use crate::receiver::{receive, ConflictingFileMode};
use crate::room_mapping::RoomIdentifier;
pub use crate::sender::send;
pub use crate::server::serve;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
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
    UnexpectedMessage,
    ServerError,
    BadRoomIdentifier,
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
