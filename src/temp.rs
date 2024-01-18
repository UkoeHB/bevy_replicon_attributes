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
//-------------------------------------------------------------------------------------------------------------------

enum UpdateType
{
    Insert,
    Remove,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Caches internal buffers for mapping attribute-based visibility to replicon's entity-based visibility.
#[derive(Resource, Default)]
pub(crate) struct VisibilityCache
{
    /// [ attribute type id : [ condition id ] ]
    attributes: HashMap<VisibilityAttributeId, HashSet<VisibilityConditionId>>,

    /// [ entity : condition ]
    entities: EntityHashMap<Entity, VisibilityConditionId>,

    /// [ condition id : (condition, [ entity ], [ client ]) ]
    conditions: HashMap<VisibilityConditionId, (VisibilityCondition, EntityHashSet<Entity>, HashSet<ClientId>)>,

    /// [ client : [ attribute type id ] ]
    clients: HashMap<ClientId, HashSet<VisibilityAttributeId>>,

    /// Attribute sets cached for use by future clients.
    /// [ attribute type id ]
    attributes_buffer: Vec<HashSet<VisibilityAttributeId>>,
}

impl VisibilityCache
{
    /// Adds an attribute to a client.
    pub(crate) fn add_client_attribute(
        &mut self,
        client_info : &mut ClientsInfo,
        client_id   : ClientId,
        attribute   : VisibilityAttributeId,
    ){
        self.update_client_visibility(client_info, client_id, attribute, UpdateType::Insert);
    }

    /// Removes an attribute from a client.
    pub(crate) fn remove_client_attribute(
        &mut self,
        client_info : &mut ClientsInfo,
        client_id   : ClientId,
        attribute   : VisibilityAttributeId,
    ){
        self.update_client_visibility(client_info, client_id, attribute, UpdateType::Remove);
    }

    /// Updates an entity's visibility condition.
    pub(crate) fn add_entity_condition(
        &mut self,
        client_info : &mut ClientsInfo,
        entity      : Entity,
        condition   : VisibilityCondition,
    ){
        // Clean up previous condition.
        let condition_id = condition.condition_id();
        if !self.remove_entity_with_check(self, client_info, entity, Some(condition_id))
        { tracing::debug!(?entity, ?condition_id, "ignoring attempt to add an entity condition that already exists"); return; }

        // Update entity map.
        self.entities.insert(entity, condition_id);

        // Update conditions map.
        let (condition, entities, clients) = self.conditions
            .entry(&condition_id)
            .or_insert_with(move || (condition, EntityHashSet::default(), HashSet::default()));

        // Add entity to tracked set for this condition.
        entities.insert(entity);

        // Update visibility of this entity for clients that can see this condition.
        for client in clients.iter()
        {
            client_info.client_mut(*client)
                .visibility_mut()
                .set_visibility(entity, true);
        }
    }

    /// Removes an entity that no longer has a replication condition.
    ///
    /// Note: We update the `ClientsInfo` in case [`Visibility`] was removed from an entity that still has
    ///       the `Replication` component.
    //todo: updating `ClientsInfo` is redundant work
    pub(crate) fn remove_entity(&mut self, client_info: &mut ClientsInfo, entity: Entity)
    {
        self.remove_entity_with_check(self, client_info, entity, None);
    }

    /// Removes a client.
    pub(crate) fn remove_client(&mut self, client_id: ClientId)
    {
        // Remove client entry
        let Some(attributes) = self.clients.remove(&client_id) else { return; };

        // Find conditions monitored by this client.
        for attribute in attributes.iter()
        {
            let Some(conditions) = self.attributes.get(attribute) else { continue; };
            for condition in conditions.iter()
            {
                let Some((_, _, clients)) = self.conditions.get_mut(condition)
                else { tracing::error!("condition missing on remove client"); continue; };

                // Clean up the client.
                // - We do not log an error on failure because this client may not have visibility of this condition.
                clients.remove(&client_id);
            }
        }

        // Cache the attributes buffer for a future client.
        self.attributes_buffer.push(attributes);
    }

    /// Accesses a client's attributes.
    pub(crate) fn client_attributes(&self, client_id: ClientId) -> Option<&HashSet<VisibilityAttributeId>>
    {
        self.clients.get(&client_id)
    }

    /// Iterates a client's attributes.
    pub(crate) fn iter_client_attributes(&self, client_id: ClientId) -> impl Iterator<Item = VisibilityAttributeId> + '_
    {
        self.clients
            .get(&client_id)
            .map(|a| a.iter().map(|a| *a))
            .into_iter()
            .flatten()
    }

    /// Updates a client's visibility relative to a specific attribute.
    fn update_client_visibility(
        &mut self, 
        client_info : &mut ClientsInfo,
        client_id   : ClientId,
        attribute   : VisibilityAttributeId,
        update      : UpdateType,
    ){
        // Access client attributes.
        let mut client_attributes = self.clients
            .entry(client_id)
            .or_insert_with(|| self.attributes_buffer.pop().or_default());

        // Update the attribute for this client.
        // - Leave if the update did nothing.
        match update
        {
            UpdateType::Insert => { if !client_attributes.insert(&attribute) { return; } }
            UpdateType::Remove => { if !client_attributes.remove(&attribute) { return; } }
        }
        //let client_attributes = &*client_attributes;  //launder the reference so we can use it again

        // Access conditions associated with this attribute.
        let Some(condition_ids) = self.attributes.get(&attribute) else { return; };

        // Get client visibility settings.
        let mut visibility_settings = client_info.client_mut(client_id).visibility_mut();

        // Update the entities and clients attached to each condition.
        for condition_id in condition_ids.iter()
        {
            let Some((condition, entities, clients)) = self.conditions.get(condition_id)
            else { tracing::error!("missing condition on update client visibility"); continue; };

            // Evaluate client visibility for this condition.
            let visibility = condition.evaluate(|a| client_attributes.contains(&a));

            // Save the client's visibility of this condition.
            match visibility
            {
                true  => clients.insert(client_id),
                false => clients.remove(client_id),
            }

            // Set visibility for entities attached to this condition.
            for entity in entities.iter()
            {
                visibility_settings.set_visibility(*entity, visibility);
            }
        }
    }

    /// Removes an entity after checking if it has a different condition than `check_condition`.
    ///
    /// Returns `false` if the check failed.
    fn remove_entity_with_check(
        &mut self,
        client_info     : &mut ClientsInfo,
        entity          : Entity,
        check_condition : Option<VisibilityConditionId>,
    ) -> bool
    {
        // Update entity map.
        let Some(condition_id) = self.entities.remove(&entity) else { return true; };

        // Check if removal was unwanted.
        if let Some(check_condition) = check_condition
        {
            if check_condition == condition_id
            {
                self.entities.insert(entity, condition_id);
                return false;
            }
        }

        // Access conditions map.
        let Some((condition, entities, clients)) = self.conditions.get_mut(&condition_id)
        else { tracing::error!("missing condition on remove entity"); return true; };

        // Remove entity from tracked set for this condition.
        if !entities.remove(entity) { tracing::error!("missing entity on remove entity"); }

        // Update visibility of this entity for clients that can see this condition.
        for client in clients.iter()
        {
            client_info.client_mut(*client)
                .visibility_mut()
                .set_visibility(entity, false);
        }

        true
    }
}

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
pub struct ClientAttributes<'w, 's>
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

    /// Accesses a client's attributes.
    pub fn attributes(&self, client_id: ClientId) -> Option<&HashSet<VisibilityAttributeId>>
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
