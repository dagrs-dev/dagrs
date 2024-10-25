use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, mpsc};

use crate::graph::node::NodeId;

use super::information_packet::Content;

/// # Output Channels
/// A hash-table mapping `NodeId` to `OutChannel`. In **Dagrs**, each `Node` stores output
/// channels in this map, enabling `Node` to send information packets to other `Node`s.
/// ## Implementaions
/// - `blocking_send_to`: call `blocking_send` of the sender by the given `NodeId`. Returns `Ok()`
/// if message sent; returns `Err(SendErr)` if the given `NodeId` is invalid or err occurs.
/// - `send_to`: receives the next value for this sender by the given `NodeId` asynchronously. Returns `Ok()`
/// if message sent; returns `Err(SendErr)` if the given `NodeId` is invalid, or err occurs.
/// - `close`: close the channel by the given `NodeId`, and remove the channel in this map.
#[derive(Default)]
pub struct OutChannels(pub HashMap<NodeId, Arc<OutChannel>>);

impl OutChannels {
    /// call `blocking_send` of the sender by the given `NodeId`. Returns `Ok()`
    /// if message sent; returns `Err(SendErr)` if the given `NodeId` is invalid, no message is available to recv,
    /// or err occurs.
    pub fn blocking_send_to(&self, id: &NodeId, content: Content) -> Result<(), SendErr> {
        match self.get(id) {
            Some(channel) => channel.blocking_send(content),
            None => Err(SendErr::ChannelNExist),
        }
    }

    /// Receives the next value for this sender by the given `NodeId` asynchronously. Returns `Ok()`
    /// if message sent; returns `Err(SendErr)` if the given `NodeId` is invalid, or no message is available to recv,
    /// or err occurs.
    pub async fn send_to(&self, id: &NodeId, content: Content) -> Result<(), SendErr> {
        match self.get(id) {
            Some(channel) => channel.send(content).await,
            None => Err(SendErr::ChannelNExist),
        }
    }

    /// Close the channel by the given `NodeId`, and remove the channel in this map.
    pub fn close(&mut self, id: &NodeId) {
        if let Some(c) = self.get(id) {
            self.0.remove(id);
        }
    }

    fn get(&self, id: &NodeId) -> Option<Arc<OutChannel>> {
        match self.0.get(id) {
            Some(c) => Some(c.clone()),
            None => None,
        }
    }
}

/// # Output Channel
/// Wrapper of senders of `tokio::sync::mpsc` and `tokio::sync::broadcast`. **Dagrs** will
/// decide the inner type of channel when building the graph.
/// ## Implements
/// - `blocking_send`: sends the message, blocked if no capacity left in the channel. Returns `Ok()`
/// if message sent; returns `Err(SendErr)` if error occurs.
/// - `send`: sends the message, waiting until there is capacity asynchronously. Returns `Ok()`
/// if message sent; returns `Err(SendErr)` if error occurs.
pub enum OutChannel {
    /// Sender of a `tokio::sync::mpsc` channel.
    Mpsc(mpsc::Sender<Content>),
    /// Sender of a `tokio::sync::broadcast` channel.
    Bcst(broadcast::Sender<Content>),
}

impl OutChannel {
    fn blocking_send(&self, value: Content) -> Result<(), SendErr> {
        match self {
            OutChannel::Mpsc(sender) => match sender.blocking_send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::MpscError(e)),
            },
            OutChannel::Bcst(sender) => match sender.send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::BcstError(e)),
            },
        }
    }

    async fn send(&self, value: Content) -> Result<(), SendErr> {
        match self {
            OutChannel::Mpsc(sender) => match sender.send(value).await {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::MpscError(e)),
            },
            OutChannel::Bcst(sender) => match sender.send(value) {
                Ok(_) => Ok(()),
                Err(e) => Err(SendErr::BcstError(e)),
            },
        }
    }
}

/// # Output Channel Error Types
/// - ChannelNExist: try to get a channel with an invalid `NodeId`.
/// - MpscError: An error related to mpsc channel.
/// - BcstError: An error related to broadcast channel.
///
/// In cases of getting errs of type `MpscError` and `BcstError`, the sender
/// will find there are no active receivers left, so try to send messages is
/// meaningless for now.
#[derive(Debug)]
pub enum SendErr {
    ChannelNExist,
    MpscError(mpsc::error::SendError<Content>),
    BcstError(broadcast::error::SendError<Content>),
}
