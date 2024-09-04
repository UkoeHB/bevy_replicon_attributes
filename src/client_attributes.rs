//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy_replicon::prelude::{ReplicatedClient, ReplicatedClients, ClientId};

//standard shortcuts
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------------------------

/// System parameter for updating client visibility attributes.
///
/// Example:
/**
```rust
#[derive(VisibilityAttribute, Default, PartialEq)]
struct IsDead;

fn kill_player(In(client_id): In<ClientId>, mut attributes: ClientAttributes)
{
    attributes.add(client_id, IsDead);
}
```
*/
#[derive(SystemParam)]
pub struct ClientAttributes<'w>
{
    cache: ResMut<'w, VisibilityCache>,
    client_info: ResMut<'w, ReplicatedClients>,
}

impl<'w> ClientAttributes<'w>
{
    /// Adds an attribute to a client.
    pub fn add<T: VisibilityAttribute>(&mut self, client_id: ClientId, attribute: T)
    {
        self.cache.add_client_attribute(&mut self.client_info, client_id, attribute.attribute_id());
    }

    /// Removes an attribute from a client.
    pub fn remove<T: VisibilityAttribute>(&mut self, client_id: ClientId, attribute: T)
    {
        self.cache.remove_client_attribute(&mut self.client_info, client_id, attribute.attribute_id());
    }

    /// Gets a client's attributes.
    pub fn get(&self, client_id: ClientId) -> Option<&HashSet<VisibilityAttributeId>>
    {
        self.cache.client_attributes(client_id)
    }

    /// Iterates a client's attributes.
    pub fn iter(&self, client_id: ClientId) -> impl Iterator<Item = VisibilityAttributeId> + '_
    {
        self.cache.iter_client_attributes(client_id)
    }

    /// Evaluates a visibility condition against all clients.
    ///
    /// Returns an iterator of clients that evaluate true.
    pub fn evaluate<'s, 'a: 's>(&'s self, condition: &'a VisibilityCondition) -> impl Iterator<Item = ClientId> + '_
    {
        self.cache.iter_client_visibility(condition)
    }

    /// Evaluates a visibility condition against connected clients.
    ///
    /// Returns an iterator of client ids and last-change ticks for clients that evaluate true.
    pub fn evaluate_connected<'s, 'a: 's>(
        &'s self,
        condition: &'a VisibilityCondition
    ) -> impl Iterator<Item = &ReplicatedClient> + '_
    {
        self.client_info
            .iter()
            .filter_map(
                |client_state|
                {
                    if !self.cache.client_visibility(client_state.id(), condition) { return None; }
                    Some(client_state)
                }
            )
    }

    /// Evaluates a visibility condition against the 'server player' if it exists.
    ///
    /// Returns `None` if the server player doesn't exist or they don't satisfy the visibility condition.
    pub fn evaluate_server_player<'s, 'a: 's>(
        &'s self,
        condition: &'a VisibilityCondition
    ) -> Option<ClientId>
    {
        let Some(server_id) = self.cache.server_id() else { return None; };

        match self.cache.client_visibility(server_id, condition)
        {
            true  => Some(server_id),
            false => None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
