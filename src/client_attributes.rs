//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientId, ClientsInfo};
use siphasher::sip128::{Hasher128, SipHasher13};
use smallvec::SmallVec;

//standard shortcuts
use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

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
    client_info: ResMut<'w, ClientsInfo>,
    cache: ResMut<'w, VisibilityCache>,
}

impl ClientAttributes
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
}

//-------------------------------------------------------------------------------------------------------------------
