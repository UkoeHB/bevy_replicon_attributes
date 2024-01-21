//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::renet::ServerEvent;
use bevy_replicon::prelude::{ClientCache, ServerSet};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn reset_clients(
    mut visibility_cache : ResMut<VisibilityCache>,
    mut client_cache     : ResMut<ClientCache>,
    mut events           : EventReader<ServerEvent>,
){
    for event in events.read()
    {
        match event
        {
            ServerEvent::ClientConnected{ client_id }        => visibility_cache.reset_client(&mut client_cache, *client_id),
            ServerEvent::ClientDisconnected{ client_id, .. } => visibility_cache.remove_client(*client_id),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn repair_clients(
    mut visibility_cache : ResMut<VisibilityCache>,
    mut client_cache     : ResMut<ClientCache>,
    mut events           : EventReader<ServerEvent>,
){
    for event in events.read()
    {
        match event
        {
            ServerEvent::ClientConnected{ client_id } => visibility_cache.repair_client(&mut client_cache, *client_id),
            ServerEvent::ClientDisconnected{ .. }     => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_visibility_removals(
    mut visibility_cache : ResMut<VisibilityCache>,
    mut client_cache     : ResMut<ClientCache>,
    mut removed          : RemovedComponents<Visibility>,
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
    mut client_cache     : ResMut<ClientCache>,
    changed              : Query<(Entity, &Visibility), Changed<Visibility>>,
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
        app.add_systems(PreUpdate, reset_clients.in_set(VisibilityReconnectSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct AttributesRepairPlugin;

impl Plugin for AttributesRepairPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PreUpdate, repair_clients.in_set(VisibilityReconnectSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// System set that collects entity [`Visibility`] changes and translates them into `bevy_replicon` client visibility.
///
/// Runs in `PostUpdate` before `bevy_replicon::prelude::ServerSet::Send`.
#[derive(SystemSet, Debug, Eq, PartialEq, Clone, Hash)]
pub struct VisibilityUpdateSet;

/// System set that handles client reconnects.
///
/// Runs in `PreUpdate` after `bevy_replicon::prelude::ServerSet::Receive`.
///
/// See [`ReconnectPolicy`] for the behavior of this set.
#[derive(SystemSet, Debug, Eq, PartialEq, Clone, Hash)]
pub struct VisibilityReconnectSet;

/// Configures handling of reconnects,
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ReconnectPolicy
{
    /// Reset a client's visibility after a disconnect and when they reconnect.
    ///
    /// Only attributes added while the client is connected will be used to determine visibility.
    Reset,
    /// Preserve client attributes after a disconnect, and repair client visibility within `bevy_replicon` when
    /// the client reconnects.
    Repair,
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that sets up visibility handling systems.
pub struct VisibilityAttributesPlugin
{
    /// See [`ReconnectPolicy`].
    pub reconnect_policy: ReconnectPolicy,
}

impl Plugin for VisibilityAttributesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(VisibilityCache::new())
            .configure_sets(PreUpdate, VisibilityReconnectSet.after(ServerSet::Receive))
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
    }
}

//-------------------------------------------------------------------------------------------------------------------
