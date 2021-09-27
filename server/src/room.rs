use std::collections::{hash_set::Iter, HashSet, VecDeque};

use super::{world_type::WorldType, user::user_key::UserKey};

#[allow(missing_docs)]
#[allow(unused_doc_comments)]
pub mod room_key {
    // The Key used to get a reference of a Room
    new_key_type! { pub struct RoomKey; }
}

pub struct Room<P: ProtocolType, W: WorldType<P>> {
    users: HashSet<UserKey>,
    entities: HashSet<W::EntityKey>,
    entity_removal_queue: VecDeque<(UserKey, W::EntityKey)>,
}

impl<P: ProtocolType, W: WorldType<P>> Room<P, W> {
    pub(crate) fn new() -> Room<P, W> {
        Room {
            users: HashSet::new(),
            entities: HashSet::new(),
            entity_removal_queue: VecDeque::new(),
        }
    }

    // Users

    pub(crate) fn has_user(&self, user_key: &UserKey) -> bool {
        return self.users.contains(user_key);
    }

    pub(crate) fn subscribe_user(&mut self, user_key: &UserKey) {
        self.users.insert(*user_key);
    }

    pub(crate) fn unsubscribe_user(&mut self, user_key: &UserKey) {
        self.users.remove(user_key);
        for entity_key in self.entities.iter() {
            self.entity_removal_queue
                .push_back((*user_key, *entity_key));
        }
    }

    pub(crate) fn user_keys(&self) -> Iter<UserKey> {
        return self.users.iter();
    }

    pub(crate) fn users_count(&self) -> usize {
        return self.users.len();
    }

    // Entities

    pub(crate) fn add_entity(&mut self, entity_key: &W::EntityKey) {
        self.entities.insert(*entity_key);
    }

    pub(crate) fn remove_entity(&mut self, entity_key: &W::EntityKey) -> bool {
        if self.entities.remove(entity_key) {
            for user_key in self.users.iter() {
                self.entity_removal_queue
                    .push_back((*user_key, *entity_key));
            }
            return true;
        } else {
            panic!("Room does not contain Entity");
        }
    }

    pub(crate) fn entity_keys(&self) -> Iter<W::EntityKey> {
        return self.entities.iter();
    }

    pub(crate) fn pop_entity_removal_queue(&mut self) -> Option<(UserKey, W::EntityKey)> {
        return self.entity_removal_queue.pop_front();
    }

    pub(crate) fn entities_count(&self) -> usize {
        return self.entities.len();
    }
}

// room references

use naia_shared::ProtocolType;

use super::server::Server;

use room_key::RoomKey;

// RoomRef

pub struct RoomRef<'s, P: ProtocolType, W: WorldType<P>> {
    server: &'s Server<P, W>,
    key: RoomKey,
}

impl<'s, P: ProtocolType, W: WorldType<P>> RoomRef<'s, P, W> {
    pub fn new(server: &'s Server<P, W>, key: &RoomKey) -> Self {
        RoomRef { server, key: *key }
    }

    pub fn key(&self) -> RoomKey {
        self.key
    }

    // Users

    pub fn has_user(&self, user_key: &UserKey) -> bool {
        return self.server.room_has_user(&self.key, user_key);
    }

    pub fn users_count(&self) -> usize {
        return self.server.room_users_count(&self.key);
    }

    // Entities

    pub fn has_entity(&self, entity_key: &W::EntityKey) -> bool {
        return self.server.room_has_entity(&self.key, entity_key);
    }

    pub fn entities_count(&self) -> usize {
        return self.server.room_entities_count(&self.key);
    }
}

// RoomMut
pub struct RoomMut<'s, P: ProtocolType, W: WorldType<P>> {
    server: &'s mut Server<P, W>,
    key: RoomKey,
}

impl<'s, P: ProtocolType, W: WorldType<P>> RoomMut<'s, P, W> {
    pub fn new(server: &'s mut Server<P, W>, key: &RoomKey) -> Self {
        RoomMut { server, key: *key }
    }

    pub fn key(&self) -> RoomKey {
        self.key
    }

    pub fn destroy(&mut self) {
        self.server.room_destroy(&self.key);
    }

    // Users

    pub fn has_user(&self, user_key: &UserKey) -> bool {
        return self.server.room_has_user(&self.key, user_key);
    }

    pub fn add_user(&mut self, user_key: &UserKey) -> &mut Self {
        self.server.room_add_user(&self.key, user_key);

        self
    }

    pub fn remove_user(&mut self, user_key: &UserKey) -> &mut Self {
        self.server.room_remove_user(&self.key, user_key);

        self
    }

    pub fn users_count(&self) -> usize {
        return self.server.room_users_count(&self.key);
    }

    // Entities

    pub fn has_entity(&self, entity_key: &W::EntityKey) -> bool {
        return self.server.room_has_entity(&self.key, entity_key);
    }

    pub fn add_entity(&mut self, entity_key: &W::EntityKey) -> &mut Self {
        self.server.room_add_entity(&self.key, entity_key);

        self
    }

    pub fn remove_entity(&mut self, entity_key: &W::EntityKey) -> &mut Self {
        self.server.room_remove_entity(&self.key, entity_key);

        self
    }

    pub fn entities_count(&self) -> usize {
        return self.server.room_entities_count(&self.key);
    }
}
