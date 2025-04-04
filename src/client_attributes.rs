//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy_replicon::prelude::ClientVisibility;
use bevy_replicon::shared::backend::connected_client::{NetworkId, NetworkIdMap};

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

fn kill_player(In(client_id): In<u64>, mut attributes: ClientAttributes)
{
    attributes.add(client_id, IsDead);
}
```
*/
#[derive(SystemParam)]
pub struct ClientAttributes<'w, 's>
{
    id_map: Res<'w, NetworkIdMap>,
    cache: ResMut<'w, VisibilityCache>,
    client_entities: Query<'w, 's, (&'static mut ClientVisibility, &'static NetworkId)>,
}

impl<'w, 's> ClientAttributes<'w, 's>
{
    /// Adds an attribute to a client.
    pub fn add<T: VisibilityAttribute>(&mut self, client_id: u64, attribute: T)
    {
        let client_entity = self.id_map.get(&NetworkId::new(client_id)).copied();
        self.cache.add_client_attribute(&mut self.client_entities.transmute_lens().query(), client_entity, client_id, attribute.attribute_id());
    }

    /// Removes an attribute from a client.
    pub fn remove<T: VisibilityAttribute>(&mut self, client_id: u64, attribute: T)
    {
        let client_entity = self.id_map.get(&NetworkId::new(client_id)).copied();
        self.cache.remove_client_attribute(&mut self.client_entities.transmute_lens().query(), client_entity, client_id, attribute.attribute_id());
    }

    /// Gets a client's attributes.
    pub fn get(&self, client_id: u64) -> Option<&HashSet<VisibilityAttributeId>>
    {
        self.cache.client_attributes(client_id)
    }

    /// Iterates a client's attributes.
    pub fn iter(&self, client_id: u64) -> impl Iterator<Item = VisibilityAttributeId> + '_
    {
        self.cache.iter_client_attributes(client_id)
    }

    /// Evaluates a visibility condition against all clients.
    ///
    /// Returns an iterator of clients that evaluate true.
    pub fn evaluate<'b, 'a: 'b>(&'b self, condition: &'a VisibilityCondition) -> impl Iterator<Item = u64> + 'b
    {
        self.cache.iter_client_visibility(condition)
    }

    /// Evaluates a visibility condition against connected clients.
    ///
    /// Returns an iterator of client ids and last-change ticks for clients that evaluate true.
    pub fn evaluate_connected<'b, 'a: 'b>(
        &'b self,
        condition: &'a VisibilityCondition
    ) -> impl Iterator<Item = u64> + 'b
    {
        self.client_entities
            .iter()
            .filter_map(
                |(_client_vis, id)|
                {
                    if !self.cache.client_visibility(id.get(), condition) { return None; }
                    Some(id.get())
                }
            )
    }

    /// Evaluates a visibility condition against the 'server player' if it exists.
    ///
    /// Returns `None` if the server player doesn't exist or they don't satisfy the visibility condition.
    pub fn evaluate_server_player<'b, 'a: 'b>(
        &'b self,
        condition: &'a VisibilityCondition
    ) -> Option<u64>
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
