use bevy_ecs::{
    entity::Entity,
    system::{Command as BevyCommand, EntityCommands},
    world::{Mut, World},
};

use naia_bevy_shared::{EntityAuthStatus, HostOwned, WorldMutType, WorldProxyMut};
use naia_client::{Client as NaiaClient, ReplicationConfig};

use crate::Client;

// Bevy Commands Extension
pub trait CommandsExt<'a> {
    fn local_duplicate(&'a mut self) -> &'a mut EntityCommands<'a>;
    fn enable_replication(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a>;
    fn disable_replication(&'a mut self, client: &mut Client)
        -> &'a mut EntityCommands<'a>;
    fn configure_replication(
        &'a mut self,
        config: ReplicationConfig,
    ) -> &'a mut EntityCommands<'a>;
    fn replication_config(&'a self, client: &Client) -> Option<ReplicationConfig>;
    fn request_authority(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a>;
    fn release_authority(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a>;
    fn authority(&'a self, client: &Client) -> Option<EntityAuthStatus>;
}

impl<'a> CommandsExt<'a> for EntityCommands<'a> {
    fn local_duplicate(&'a mut self) -> &'a mut EntityCommands<'a> {
        let old_entity = self.id();
        let mut commands = self.commands();
        let new_entity = commands.spawn_empty().id();
        let command = LocalDuplicateComponents::new(new_entity, old_entity);
        commands.add(command);
        commands.entity(new_entity);
        return self;
    }

    fn enable_replication(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a> {
        client.enable_replication(&self.id());
        self.insert(HostOwned);
        return self;
    }

    fn disable_replication(
        &'a mut self,
        client: &mut Client,
    ) -> &'a mut EntityCommands<'a> {
        client.disable_replication(&self.id());
        self.remove::<HostOwned>();
        return self;
    }

    fn configure_replication(
        &'a mut self,
        config: ReplicationConfig,
    ) -> &'a mut EntityCommands<'a> {
        let entity = self.id();
        let mut commands = self.commands();
        let command = ConfigureReplicationCommand::new(entity, config);
        commands.add(command);
        return self;
    }

    fn replication_config(&'a self, client: &Client) -> Option<ReplicationConfig> {
        client.replication_config(&self.id())
    }

    fn request_authority(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a> {
        client.entity_request_authority(&self.id());
        return self;
    }

    fn release_authority(&'a mut self, client: &mut Client) -> &'a mut EntityCommands<'a> {
        client.entity_release_authority(&self.id());
        return self;
    }

    fn authority(&'a self, client: &Client) -> Option<EntityAuthStatus> {
        client.entity_authority_status(&self.id())
    }
}

//// LocalDuplicateComponents Command ////
pub(crate) struct LocalDuplicateComponents {
    mutable_entity: Entity,
    immutable_entity: Entity,
}

impl LocalDuplicateComponents {
    pub fn new(new_entity: Entity, old_entity: Entity) -> Self {
        Self {
            mutable_entity: new_entity,
            immutable_entity: old_entity,
        }
    }
}

impl BevyCommand for LocalDuplicateComponents {
    fn apply(self, world: &mut World) {
        WorldMutType::<Entity>::local_duplicate_components(
            &mut world.proxy_mut(),
            &self.mutable_entity,
            &self.immutable_entity,
        );
    }
}

//// ConfigureReplicationCommand Command ////
pub(crate) struct ConfigureReplicationCommand {
    entity: Entity,
    config: ReplicationConfig,
}

impl ConfigureReplicationCommand {
    pub fn new(entity: Entity, config: ReplicationConfig) -> Self {
        Self { entity, config }
    }
}

impl BevyCommand for ConfigureReplicationCommand {
    fn apply(self, world: &mut World) {
        world.resource_scope(|world, mut client: Mut<NaiaClient<Entity>>| {
            client.configure_entity_replication(&mut world.proxy_mut(), &self.entity, self.config);
        });
    }
}
