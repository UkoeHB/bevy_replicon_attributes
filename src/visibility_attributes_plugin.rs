//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientId, ReplicatedClients, ClientConnected, ClientDisconnected, ServerSet, VisibilityPolicy};
use bevy_replicon::server::StartReplication;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_server_to_cache(server_id: ClientId) -> impl FnMut(ResMut<'_, VisibilityCache>, ResMut<'_, ReplicatedClients>)
{
    move
    |
        mut visibility_cache : ResMut<VisibilityCache>,
        mut client_cache     : ResMut<ReplicatedClients>
    |
    {
        visibility_cache.add_server_as_client(&mut client_cache, server_id);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_connected(
    event: Trigger<ClientConnected>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_cache: ResMut<ReplicatedClients>,
){
    let ClientConnected{ client_id } = event.event();
    visibility_cache.remove_client(*client_id);

    // If StartReplication won't be sent then we can reset the client right now.
    if client_cache.replicate_after_connect() {
        visibility_cache.reset_client(&mut client_cache, *client_id);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_disconnected(
    event: Trigger<ClientDisconnected>,
    mut visibility_cache: ResMut<VisibilityCache>,
){
    let ClientDisconnected{ client_id, .. } = event.event();
    visibility_cache.remove_client(*client_id);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_start_replication(
    event: Trigger<StartReplication>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_cache: ResMut<ReplicatedClients>,
){
    let StartReplication(client_id) = event.event();
    // We can't do this at the same time as connecting since replication may not start right away when relying
    // on StartReplication.
    visibility_cache.reset_client(&mut client_cache, *client_id);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn repair_clients(
    event: Trigger<StartReplication>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_cache: ResMut<ReplicatedClients>,
){
    let StartReplication(client_id) = event.event();
    // This will load visibility settings into replicon, which clears visibility when a client disconnects.
    visibility_cache.repair_client(&mut client_cache, *client_id);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_visibility_removals(
    mut visibility_cache : ResMut<VisibilityCache>,
    mut client_cache     : ResMut<ReplicatedClients>,
    mut removed          : RemovedComponents<VisibilityCondition>,
){
    for entity in removed.read()
    {
        visibility_cache.remove_entity(&mut client_cache, entity);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_visibility_changes(
    mut visibility_cache : ResMut<VisibilityCache>,
    mut client_cache     : ResMut<ReplicatedClients>,
    changed              : Query<(Entity, &VisibilityCondition), Changed<VisibilityCondition>>,
){
    for (entity, visibility) in changed.iter()
    {
        visibility_cache.add_entity_condition(&mut client_cache, entity, &*visibility);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct AttributesResetPlugin;

impl Plugin for AttributesResetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(reset_clients_on_connected)
            .add_observer(reset_clients_on_disconnected)
            .add_observer(reset_clients_on_start_replication);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct AttributesRepairPlugin;

impl Plugin for AttributesRepairPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(repair_clients);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// System set that collects entity [`VisibilityCondition`] changes and translates them into `bevy_replicon` client
/// visibility.
///
/// Runs in `PostUpdate` before `bevy_replicon::prelude::ServerSet::Send`.
#[derive(SystemSet, Debug, Eq, PartialEq, Clone, Hash)]
pub struct VisibilityUpdateSet;

/// Configures handling of reconnects,
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ReconnectPolicy
{
    /// Resets a client's visibility when they connect and after a disconnect.
    ///
    /// Only attributes added while the client is connected will be used to determine visibility.
    ///
    /// Newly-connected clients always start with the builtin [`Global`] and [`Client`] attributes.
    Reset,
    /// Preserves client attributes after a disconnect, and repairs client visibility within `bevy_replicon` when
    /// the client reconnects.
    ///
    /// Attributes can be added to clients at any time, even before they connect for the first time.
    ///
    /// Newly-connected clients always start with the builtin [`Global`] and [`Client`] attributes.
    Repair,
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that sets up visibility handling systems in a server using `bevy_replicon`.
pub struct VisibilityAttributesPlugin
{
    /// Records the server's `ClientId` if it is a player.
    ///
    /// This needs to be set if you want events sent via [`ServerEventSender`] to be echoed to the server.
    pub server_id: Option<ClientId>,
    /// See [`ReconnectPolicy`].
    pub reconnect_policy: ReconnectPolicy,
}

impl Plugin for VisibilityAttributesPlugin
{
    fn build(&self, app: &mut App)
    {
        // todo: replace with plugin dependencies if bevy adds them ??
        let cache = app
            .world()
            .get_resource::<ReplicatedClients>()
            .expect("bevy_replicon plugins are required for VisibilityAttributesPlugin");
        if let VisibilityPolicy::Blacklist = cache.visibility_policy()
        {
            panic!("bevy_replicon VisibilityPolicy::Blacklist is not compatible with VisibilityAttributesPlugin, use \
                VisibilityPolicy::Whitelist instead");
        }

        app.insert_resource(VisibilityCache::new())
            .configure_sets(PostUpdate, VisibilityUpdateSet.before(ServerSet::Send))
            .add_systems(PostUpdate,
                (
                    // handle removals first in case of removal -> insertion in different systems
                    handle_visibility_removals,
                    handle_visibility_changes,
                )
                    .chain()
                    .in_set(VisibilityUpdateSet)
            );

        match self.reconnect_policy
        {
            ReconnectPolicy::Reset  => { app.add_plugins(AttributesResetPlugin); }
            ReconnectPolicy::Repair => { app.add_plugins(AttributesRepairPlugin); }
        }

        if let Some(server_id) = self.server_id
        {
            app.add_systems(Startup, add_server_to_cache(server_id));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
