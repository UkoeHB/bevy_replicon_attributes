//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientVisibility, ServerSet, VisibilityPolicy};
use bevy_replicon::server::ServerPlugin;
use bevy_replicon::shared::backend::connected_client::{NetworkId, NetworkIdMap};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct NeedsVisibilityReset;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_server_to_cache(server_id: u64) -> impl IntoSystem<(), (), ()>
{
    IntoSystem::into_system(move
    |
        mut visibility_cache: ResMut<VisibilityCache>,
        mut client_entities: Query<&mut ClientVisibility>
    |
    {
        visibility_cache.add_server_as_client(&mut client_entities, server_id);
    })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients_on_disconnected(
    event: Trigger<OnRemove, NetworkId>,
    mut visibility_cache: ResMut<VisibilityCache>,
    client_ids: Query<&NetworkId>,
){
    let client_entity = event.target();
    let Ok(client_id) = client_ids.get(client_entity) else { return };
    visibility_cache.remove_client(client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Replication starts when `ReplicatedClient` is added, but we react to `ClientVisibility` because we need that
/// component.
fn reset_clients_on_start_replication(
    event: Trigger<OnAdd, ClientVisibility>,
    mut c: Commands,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_entities: Query<&mut ClientVisibility>,
    client_ids: Query<&NetworkId>,
){
    let client_entity = event.target();
    let Ok(client_id) = client_ids.get(client_entity) else {
        // Need to do this because there is a race condition between adding ClientVisibility and adding NetworkId.
        c.entity(client_entity).insert(NeedsVisibilityReset);
        return;
    };
    c.entity(client_entity).remove::<NeedsVisibilityReset>();
    visibility_cache.reset_client(&mut client_entities, Some(client_entity), client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn fallback_reset_clients_on_network_id(
    event: Trigger<OnAdd, NetworkId>,
    mut c: Commands,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_entities: Query<&mut ClientVisibility>,
    client_ids: Query<&NetworkId, With<NeedsVisibilityReset>>,
){
    let client_entity = event.target();
    let Ok(client_id) = client_ids.get(client_entity) else { return };
    c.entity(client_entity).remove::<NeedsVisibilityReset>();
    visibility_cache.reset_client(&mut client_entities, Some(client_entity), client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn repair_clients(
    event: Trigger<OnAdd, ClientVisibility>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_entities: Query<&mut ClientVisibility>,
    client_ids: Query<&NetworkId>,
){
    let client_entity = event.target();
    let Ok(client_id) = client_ids.get(client_entity) else { return };
    // This will load visibility settings into replicon, which clears visibility when a client disconnects.
    visibility_cache.repair_client(&mut client_entities, Some(client_entity), client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_visibility_removals(
    id_map: Res<NetworkIdMap>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_entities: Query<&mut ClientVisibility>,
    mut removed: RemovedComponents<VisibilityCondition>,
){
    for entity in removed.read()
    {
        visibility_cache.remove_entity(&id_map, &mut client_entities, entity);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_visibility_changes(
    id_map: Res<NetworkIdMap>,
    mut visibility_cache: ResMut<VisibilityCache>,
    mut client_entities: Query<&mut ClientVisibility>,
    changed: Query<(Entity, &VisibilityCondition), Changed<VisibilityCondition>>,
){
    for (entity, visibility) in changed.iter()
    {
        visibility_cache.add_entity_condition(&id_map, &mut client_entities, entity, &*visibility);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct AttributesResetPlugin;

impl Plugin for AttributesResetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(reset_clients_on_disconnected)
            .add_observer(reset_clients_on_start_replication)
            .add_observer(fallback_reset_clients_on_network_id);
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
    /// Resets a client's visibility when they start replicating and after a disconnect.
    ///
    /// Only attributes added while the client is replicating will be used to determine visibility.
    ///
    /// Newly-connected clients always start with the builtin [`Global`] and [`Client`] attributes.
    Reset,
    /// Preserves client attributes after a disconnect, and repairs client visibility within `bevy_replicon` when
    /// the client reconnects and starts replicating again.
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
    /// Records the server's client id if it is a player.
    ///
    /// This needs to be set if you want events sent via [`ServerEventSender`] to be echoed to the server.
    pub server_id: Option<u64>,
    /// See [`ReconnectPolicy`].
    pub reconnect_policy: ReconnectPolicy,
}

impl Plugin for VisibilityAttributesPlugin
{
    fn build(&self, app: &mut App)
    {
        // todo: replace with plugin dependencies if bevy adds them ??
        let added_plugins = app.get_added_plugins::<ServerPlugin>();
        let server_plugin = added_plugins
            .get(0)
            .expect("bevy_replicon server plugins are required for VisibilityAttributesPlugin");
        if let VisibilityPolicy::Blacklist = server_plugin.visibility_policy
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
