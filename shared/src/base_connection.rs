use std::net::SocketAddr;

use naia_serde::{BitWriter, Serde};

use naia_socket_shared::Timer;

use super::{
    ack_manager::AckManager,
    channel_config::{ChannelConfig, ChannelIndex},
    connection_config::ConnectionConfig,
    message_manager::MessageManager,
    packet_notifiable::PacketNotifiable,
    packet_type::PacketType,
    protocolize::Protocolize,
    standard_header::StandardHeader,
    types::PacketIndex,
};

/// Represents a connection to a remote host, and provides functionality to
/// manage the connection and the communications to it
pub struct BaseConnection<P: Protocolize, C: ChannelIndex> {
    pub address: SocketAddr,
    heartbeat_timer: Timer,
    timeout_timer: Timer,
    ack_manager: AckManager,
    pub message_manager: MessageManager<P, C>,
}

impl<P: Protocolize, C: ChannelIndex> BaseConnection<P, C> {
    /// Create a new BaseConnection, given the appropriate underlying managers
    pub fn new(
        address: SocketAddr,
        connection_config: &ConnectionConfig,
        channel_config: &ChannelConfig<C>,
    ) -> Self {
        return BaseConnection {
            address,
            heartbeat_timer: Timer::new(connection_config.heartbeat_interval),
            timeout_timer: Timer::new(connection_config.disconnection_timeout_duration),
            ack_manager: AckManager::new(),
            message_manager: MessageManager::new(channel_config),
        };
    }

    // Heartbeats

    /// Record that a message has been sent (to prevent needing to send a
    /// heartbeat)
    pub fn mark_sent(&mut self) {
        return self.heartbeat_timer.reset();
    }

    /// Returns whether a heartbeat message should be sent
    pub fn should_send_heartbeat(&self) -> bool {
        return self.heartbeat_timer.ringing();
    }

    // Timeouts

    /// Record that a message has been received from a remote host (to prevent
    /// disconnecting from the remote host)
    pub fn mark_heard(&mut self) {
        return self.timeout_timer.reset();
    }

    /// Returns whether this connection should be dropped as a result of a
    /// timeout
    pub fn should_drop(&self) -> bool {
        return self.timeout_timer.ringing();
    }

    // Acks & Headers

    /// Process an incoming packet, pulling out the packet index number to keep
    /// track of the current RTT, and sending the packet to the AckManager to
    /// handle packet notification events
    pub fn process_incoming_header(
        &mut self,
        header: &StandardHeader,
        packet_notifiable: &mut Option<&mut dyn PacketNotifiable>,
    ) {
        self.ack_manager.process_incoming_header(
            &header,
            &mut self.message_manager,
            packet_notifiable,
        );
    }

    /// Given a packet payload, start tracking the packet via it's index, attach
    /// the appropriate header, and return the packet's resulting underlying
    /// bytes
    pub fn write_outgoing_header(&mut self, packet_type: PacketType, writer: &mut BitWriter) {
        // Add header onto message!
        self.ack_manager
            .next_outgoing_packet_header(packet_type)
            .ser(writer);
    }

    /// Get the next outgoing packet's index
    pub fn next_packet_index(&self) -> PacketIndex {
        return self.ack_manager.next_sender_packet_index();
    }
}
