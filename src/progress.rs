use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use crate::room_mapping::RoomIdentifier;
use crate::{CipherType, IrisError};

#[derive(Debug)]
pub enum SenderProgressMessage {
    AssignedRoomIdentifier {
        room_identifier: RoomIdentifier,
    },
    SetCipher {
        cipher_type: CipherType,
    },
    TransferMetadata {
        total_files: usize,
        total_bytes: u64,
    },
    FileMetadata {
        filename: PathBuf,
        file_size: u64,
    },
    ChunkSent {
        size: u64,
    },
    FileDone,
    DirectoryCreated,
    FileSkipped,
    Error(IrisError),
}

#[derive(Debug)]
pub enum ReceiverProgressMessage {
    SetCipher {
        cipher_type: CipherType,
    },
    TransferMetadata {
        total_files: usize,
        total_bytes: u64,
    },
    FileMetadata {
        filename: PathBuf,
        file_size: u64,
    },
    ChunkReceived {
        size: u64,
    },
    FileDone,
    DirectoryCreated,
    FileSkipped,
    Error(IrisError),
}

#[derive(Debug)]
pub enum WorkerMessage {
    Cancel,
}

pub struct SenderWorkerCommunication {
    tx_channel: Sender<WorkerMessage>,
    rx_channel: Receiver<SenderProgressMessage>,
}

pub struct ReceiverWorkerCommunication {
    tx_channel: Sender<WorkerMessage>,
    rx_channel: Receiver<ReceiverProgressMessage>,
}

pub struct SenderProgressCommunication {
    tx_channel: Sender<SenderProgressMessage>,
    rx_channel: Receiver<WorkerMessage>,
}

pub struct ReceiverProgressCommunication {
    tx_channel: Sender<ReceiverProgressMessage>,
    rx_channel: Receiver<WorkerMessage>,
}

impl SenderWorkerCommunication {
    pub fn read(&self) -> Result<Option<SenderProgressMessage>, IrisError> {
        match self.rx_channel.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(IrisError::UserConnectionReadError),
        }
    }

    pub fn write(&self, message: WorkerMessage) -> Result<(), IrisError> {
        self.tx_channel.send(message).unwrap();
        Ok(())
    }
}

impl ReceiverWorkerCommunication {
    pub fn read(&self) -> Result<Option<ReceiverProgressMessage>, IrisError> {
        match self.rx_channel.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(IrisError::UserConnectionReadError),
        }
    }

    pub fn write(&self, message: WorkerMessage) -> Result<(), IrisError> {
        self.tx_channel.send(message).unwrap();
        Ok(())
    }
}

impl SenderProgressCommunication {
    pub fn read(&self) -> Result<Option<WorkerMessage>, IrisError> {
        match self.rx_channel.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(IrisError::UserConnectionReadError),
        }
    }

    pub fn write(&self, message: SenderProgressMessage) -> Result<(), IrisError> {
        self.tx_channel.send(message).unwrap();
        Ok(())
    }
}

impl ReceiverProgressCommunication {
    pub fn read(&self) -> Result<Option<WorkerMessage>, IrisError> {
        match self.rx_channel.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(IrisError::UserConnectionReadError),
        }
    }

    pub fn write(&self, message: ReceiverProgressMessage) -> Result<(), IrisError> {
        self.tx_channel.send(message).unwrap();
        Ok(())
    }
}

pub fn get_sender_communication_channels(
) -> (SenderWorkerCommunication, SenderProgressCommunication) {
    let (worker_sender, worker_receiver) = channel();
    let (progress_sender, progress_receiver) = channel();

    let worker_communication = SenderWorkerCommunication {
        tx_channel: worker_sender,
        rx_channel: progress_receiver,
    };

    let progress_communication = SenderProgressCommunication {
        tx_channel: progress_sender,
        rx_channel: worker_receiver,
    };

    (worker_communication, progress_communication)
}

pub fn get_receiver_communication_channels(
) -> (ReceiverWorkerCommunication, ReceiverProgressCommunication) {
    let (worker_sender, worker_receiver) = channel();
    let (progress_sender, progress_receiver) = channel();

    let worker_communication = ReceiverWorkerCommunication {
        tx_channel: worker_sender,
        rx_channel: progress_receiver,
    };

    let progress_communication = ReceiverProgressCommunication {
        tx_channel: progress_sender,
        rx_channel: worker_receiver,
    };

    (worker_communication, progress_communication)
}
