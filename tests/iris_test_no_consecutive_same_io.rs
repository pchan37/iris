use std::thread;

use iris::iris_channel_stream::{IrisChannelStream, MessageTracker};
use iris::{
    get_receiver_communication_channels, get_sender_communication_channels, receive, send,
    CipherType, ConflictingFileMode,
};

/// Checks that neither sender nor receiver does consecutive read or consecutive write.
#[test]
fn test_no_consecutive_same_io() {
    let (sender_tx, receiver_rx) = std::sync::mpsc::channel();
    let (receiver_tx, sender_rx) = std::sync::mpsc::channel();

    let mut sender_connection = IrisChannelStream {
        rx_channel: sender_rx,
        tx_channel: sender_tx,
        messages_sent: vec![],
    };

    let mut receiver_connection = IrisChannelStream {
        rx_channel: receiver_rx,
        tx_channel: receiver_tx,
        messages_sent: vec![],
    };

    thread::scope(|s| {
        s.spawn(|| {
            let (_worker_communication, progress_communication) = get_sender_communication_channels();
            let files = vec!["./tests/bbb".into()];
            send(
                &mut sender_connection,
                3000,
                "this-is-secret",
                CipherType::XChaCha20Poly1305,
                files,
                &progress_communication,
            )
            .unwrap();
        });
        s.spawn(|| {
            let (_worker_communication, progress_communication) = get_receiver_communication_channels();
            receive(
                &mut receiver_connection,
                3000,
                "this-is-secret",
                ConflictingFileMode::Error,
                &progress_communication,
            )
            .unwrap();
        });
    });

    let sender_messages_sent = sender_connection.messages_sent.clone();
    let receiver_messages_sent = receiver_connection.messages_sent.clone();

    check_no_consecutive_io(sender_messages_sent);
    check_no_consecutive_io(receiver_messages_sent);

    std::fs::remove_file("bbb").unwrap();
}

fn check_no_consecutive_io(messages_sent: Vec<MessageTracker>) {
    for (index, message) in messages_sent[1..].iter().enumerate() {
        let previous_message = &messages_sent[index];
        let consecutive_same_io = match *message {
            MessageTracker::ReadBytes(_) | MessageTracker::ReadIrisMessage(_) => {
                matches!(
                    previous_message,
                    MessageTracker::ReadBytes(_) | MessageTracker::ReadIrisMessage(_)
                )
            }
            MessageTracker::WriteBytes(_) | MessageTracker::WriteIrisMessage(_) => {
                matches!(
                    previous_message,
                    MessageTracker::WriteBytes(_) | MessageTracker::WriteIrisMessage(_)
                )
            }
        };

        assert!(
            !consecutive_same_io,
            "previous message was {previous_message:?}, current message is {message:?}"
        );
    }
}
