use std::collections::VecDeque;

use crate::{protocol::protocolize::Protocolize, sequence_less_than, types::MessageId};

use super::{
    channel_config::{ChannelIndex, ReliableSettings},
    message_channel::{MessageChannel, OutgoingReliableChannel},
};

pub struct OrderedReliableChannel<P: Protocolize, C: ChannelIndex> {
    channel_index: C,
    incoming_message_id: MessageId,
    incoming_message_buffer: VecDeque<(MessageId, Option<P>)>,
    outgoing_channel: OutgoingReliableChannel<P, C>,
}

impl<P: Protocolize, C: ChannelIndex> OrderedReliableChannel<P, C> {
    pub fn new(channel_index: C, reliable_settings: &ReliableSettings) -> Self {
        Self {
            channel_index: channel_index.clone(),
            outgoing_channel: OutgoingReliableChannel::new(
                channel_index.clone(),
                reliable_settings,
            ),
            incoming_message_id: 0,
            incoming_message_buffer: VecDeque::new(),
        }
    }
}

impl<P: Protocolize, C: ChannelIndex> MessageChannel<P, C> for OrderedReliableChannel<P, C> {
    fn send_message(&mut self, message: P) {
        return self.outgoing_channel.send_message(message);
    }

    fn recv_message(&mut self, message_id: MessageId, message: P) {
        // moving from oldest incoming message to newest
        // compare existing slots and see if the message_id has been instantiated
        // already if it has, put the message into the slot
        // otherwise, keep track of what the last message id was
        // then add new empty slots at the end until getting to the incoming message id
        // then, once you're there, put the new message in

        if sequence_less_than(message_id, self.incoming_message_id) {
            // already moved sliding window past this message id
            return;
        }

        let mut index = 0;
        let mut found = false;

        loop {
            if index < self.incoming_message_buffer.len() {
                if let Some((old_message_id, _)) = self.incoming_message_buffer.get(index) {
                    if *old_message_id == message_id {
                        found = true;
                    }
                }

                if found {
                    let (_, old_message) = self.incoming_message_buffer.get_mut(index).unwrap();
                    if old_message.is_none() {
                        *old_message = Some(message);
                    } else {
                        // already received this message
                    }
                    break;
                }
            } else {
                let next_message_id = self.incoming_message_id.wrapping_add(index as u16);

                if next_message_id == message_id {
                    self.incoming_message_buffer
                        .push_back((next_message_id, Some(message)));
                    break;
                } else {
                    self.incoming_message_buffer
                        .push_back((next_message_id, None));
                }
            }

            index += 1;
        }
    }

    fn collect_outgoing_messages(
        &mut self,
        rtt_millis: &f32,
        outgoing_messages: &mut VecDeque<(C, MessageId, P)>,
    ) {
        return self
            .outgoing_channel
            .generate_messages(rtt_millis, outgoing_messages);
    }

    fn collect_incoming_messages(&mut self, incoming_messages: &mut VecDeque<(C, P)>) {
        loop {
            let mut has_message = false;
            if let Some((_, Some(_))) = self.incoming_message_buffer.front() {
                has_message = true;
            }
            if has_message {
                let (_, message_opt) = self.incoming_message_buffer.pop_front().unwrap();
                let message = message_opt.unwrap();
                incoming_messages.push_back((self.channel_index.clone(), message));
                self.incoming_message_id = self.incoming_message_id.wrapping_add(1);
            } else {
                break;
            }
        }
    }

    fn notify_message_delivered(&mut self, message_id: &MessageId) {
        return self.outgoing_channel.notify_message_delivered(message_id);
    }
}
