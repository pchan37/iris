use std::iter::zip;
use std::thread;

use iris::iris_channel_stream::{IrisChannelStream, MessageTracker};
use iris::{receive, send, CipherType, ConflictingFileMode};

/// Checks that for every read, the other party does a corresponding a write and vice versa.
///
/// Example: If sender reads an iris_message, the receiver writes an iris message.
#[test]
fn test_matching_message() {
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
            let files = vec!["./tests/aaa".into()];
            send(
                &mut sender_connection,
                2000,
                "this-is-secret",
                CipherType::XChaCha20Poly1305,
                files,
            )
            .unwrap();
        });
        s.spawn(|| {
            receive(
                &mut receiver_connection,
                2000,
                "this-is-secret",
                ConflictingFileMode::Error,
            )
            .unwrap();
        });
    });

    let sender_messages_sent = sender_connection.messages_sent.clone();
    let receiver_messages_sent = receiver_connection.messages_sent.clone();

    for (sender_message, receiver_message) in zip(sender_messages_sent, receiver_messages_sent) {
        let message_match = match sender_message {
            MessageTracker::ReadIrisMessage(message) => {
                receiver_message == MessageTracker::WriteIrisMessage(message)
            }
            MessageTracker::WriteIrisMessage(message) => {
                receiver_message == MessageTracker::ReadIrisMessage(message)
            }
            MessageTracker::ReadBytes(ref message) => {
                receiver_message == MessageTracker::WriteBytes(message.to_vec())
            }
            MessageTracker::WriteBytes(ref message) => {
                receiver_message == MessageTracker::ReadBytes(message.to_vec())
            }
        };

        assert!(
            message_match,
            "sender did a {sender_message:?}, but receiver did a {receiver_message:?}"
        );
    }

    std::fs::remove_file("aaa").unwrap();
}
