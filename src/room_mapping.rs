use std::collections::hash_map::Entry;
use std::collections::HashMap;

use rand::Rng;

use crate::iris_stream::EncryptedIrisStream;

pub type RoomIdentifier = u16;

#[derive(Debug, Default)]
pub struct RoomMapping {
    rooms: HashMap<RoomIdentifier, Box<dyn EncryptedIrisStream>>,
}

impl RoomMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_socket(&mut self, socket: impl EncryptedIrisStream + 'static) -> RoomIdentifier {
        let mut rng = rand::thread_rng();
        loop {
            let room_identifier = rng.gen_range(1000..=9999);
            if let Entry::Vacant(entry) = self.rooms.entry(room_identifier) {
                entry.insert(Box::new(socket));
                return room_identifier;
            }
        }
    }

    pub fn get_and_remove_socket(
        &mut self,
        room_identifier: RoomIdentifier,
    ) -> Option<Box<dyn EncryptedIrisStream>> {
        self.rooms.remove(&room_identifier)
    }
}
