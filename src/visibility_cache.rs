//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::utils::{EntityHashMap, EntityHashSet};
use bevy_replicon::renet::ClientId;
use bevy_replicon::prelude::ClientCache;

//standard shortcuts
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

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
#[derive(Resource)]
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

    /// Condition id sets cached for future use.
    condition_ids_buffer: Vec<HashSet<VisibilityConditionId>>,
    /// Entity sets cached for use by future clients.
    entities_buffer: Vec<EntityHashSet<Entity>>,
    /// Client id sets cached for use by future clients.
    client_ids_buffer: Vec<HashSet<ClientId>>,
    /// Attribute id sets cached for use by future clients.
    attribute_ids_buffer: Vec<HashSet<VisibilityAttributeId>>,
}

impl VisibilityCache
{
    /// Adds an attribute to a client.
    pub(crate) fn add_client_attribute(
        &mut self,
        client_cache : &mut ClientCache,
        client_id    : ClientId,
        attribute    : VisibilityAttributeId,
    ){
        self.update_client_visibility(client_cache, client_id, attribute, UpdateType::Insert);
    }

    /// Removes an attribute from a client.
    pub(crate) fn remove_client_attribute(
        &mut self,
        client_cache : &mut ClientCache,
        client_id    : ClientId,
        attribute    : VisibilityAttributeId,
    ){
        self.update_client_visibility(client_cache, client_id, attribute, UpdateType::Remove);
    }

    /// Repairs a client by refreshing visibility of all entities in the `ClientCache`.
    ///
    /// Panics if the client is not connected.
    pub(crate) fn repair_client(&mut self, client_cache: &mut ClientCache, client_id: ClientId)
    {
        // Access client attributes.
        let client_attributes = self.clients
            .entry(client_id)
            .or_insert_with(|| self.attribute_ids_buffer.pop().unwrap_or_default());

        // Get client visibility settings.
        let visibility_settings = client_cache.client_mut(client_id).visibility_mut();

        // Iterate all client attributes
        for client_attribute in client_attributes.iter()
        {
            // Access conditions associated with this attribute.
            let Some(condition_ids) = self.attributes.get(&client_attribute) else { continue; };

            // Update the entity and client sets attached to each condition.
            for condition_id in condition_ids.iter()
            {
                let Some((condition, entities, clients)) = self.conditions.get_mut(condition_id)
                else { tracing::error!("missing condition on repair client visibility"); continue; };

                // Evaluate client visibility for this condition.
                let visibility = condition.evaluate(|a| client_attributes.contains(&a));

                // Save the client's visibility of this condition.
                match visibility
                {
                    true  => { clients.insert(client_id); }
                    false => { clients.remove(&client_id); }
                }

                // Set visibility for entities attached to this condition.
                // - Ignore disconnected clients.
                tracing::trace!(?client_id, ?entities, "visibility {visibility}");

                for entity in entities.iter()
                {
                    visibility_settings.set_visibility(*entity, visibility);
                }
            }
        }
    }

    /// Updates an entity's visibility condition.
    pub(crate) fn add_entity_condition(
        &mut self,
        client_cache : &mut ClientCache,
        entity       : Entity,
        condition    : &VisibilityCondition,
    ){
        // Clean up previous condition.
        let condition_id = condition.condition_id();
        if !self.remove_entity_with_check(client_cache, entity, Some(condition_id))
        { tracing::debug!(?entity, ?condition_id, "ignoring attempt to add an entity condition that already exists"); return; }

        // Update entity map.
        if self.entities.insert(entity, condition_id).is_some()
        { tracing::error!(?entity, ?condition_id, "entity unexpectedly had condition on insert"); return; }
        tracing::trace!(?entity, ?condition_id, "added condition to entity");

        // Access conditions map.
        let entry = self.conditions.entry(condition_id);

        // Update attributes map if this is a new condition.
        if let Entry::Vacant(_) = &entry
        {
            for attribute_id in condition.iter_attributes()
            {
                if !self.attributes
                    .entry(attribute_id)
                    .or_insert_with(|| self.condition_ids_buffer.pop().unwrap_or_default())
                    .insert(condition_id)
                { tracing::error!(?attribute_id, ?condition_id, "found condition in attributes map without conditions entry"); }
            }
        }

        // Update conditions map.
        let (_, entities, clients) = entry
            .or_insert_with(
                ||
                {
                    (
                        condition.clone(),
                        self.entities_buffer.pop().unwrap_or_default(),
                        self.client_ids_buffer.pop().unwrap_or_default(),
                    )
                }
            );

        // Add entity to tracked set for this condition.
        if !entities.insert(entity)
        { tracing::error!(?entity, ?condition_id, "entity unexpectedly in tracked entities for condition"); }

        // Update visibility of this entity for clients that can see this condition.
        // - Note that it's possible we are setting client visibilities back to `true` that were set to `false`
        //   when cleaning up this entity's old condition. We assume that performance is about equivalent between
        //   this brute-force approach and trying to only modify clients that don't have visibility of both conditions.
        for client_id in clients.iter()
        {
            tracing::trace!(?client_id, ?entity, "visibility true");
            let Some(client) = client_cache.get_client_mut(*client_id) else { continue; };
            client.visibility_mut().set_visibility(entity, true);
        }
    }

    /// Removes an entity that no longer has a replication condition.
    ///
    /// Note: We update the `ClientCache` in case [`Visibility`] was removed from an entity that still has
    ///       the `Replication` component.
    //todo: updating `ClientCache` is redundant work
    pub(crate) fn remove_entity(&mut self, client_cache: &mut ClientCache, entity: Entity)
    {
        self.remove_entity_with_check(client_cache, entity, None);
    }

    /// Removes a client.
    pub(crate) fn remove_client(&mut self, client_id: ClientId)
    {
        // Remove client entry
        let Some(mut attributes) = self.clients.remove(&client_id) else { return; };

        // Find conditions monitored by this client.
        for attribute in attributes.iter()
        {
            // We do not log an error on failure because the client may have an attribute with no corresponding conditions.
            let Some(conditions) = self.attributes.get(attribute)else { continue; };

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
        attributes.clear();
        self.attribute_ids_buffer.push(attributes);
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
        client_cache : &mut ClientCache,
        client_id    : ClientId,
        attribute    : VisibilityAttributeId,
        update       : UpdateType,
    ){
        // Access client attributes.
        let client_attributes = self.clients
            .entry(client_id)
            .or_insert_with(|| self.attribute_ids_buffer.pop().unwrap_or_default());

        // Update the attribute for this client.
        // - Leave if the update did nothing.
        match update
        {
            UpdateType::Insert =>
            {
                if !client_attributes.insert(attribute)
                { tracing::debug!(?client_id, ?attribute, "ignoring inserted client attribute that already exists"); return; }
                tracing::trace!(?client_id, ?attribute, "inserted attribute to client");
            }
            UpdateType::Remove =>
            {
                if !client_attributes.remove(&attribute)
                { tracing::debug!(?client_id, ?attribute, "ignoring remove client attribute that doesn't exist"); return; }
                tracing::trace!(?client_id, ?attribute, "removed attribute from client");
            }
        }

        // Access conditions associated with this attribute.
        let Some(condition_ids) = self.attributes.get(&attribute) else { return; };

        // Get client visibility settings.
        let mut visibility_settings = client_cache.get_client_mut(client_id).map(|c| c.visibility_mut());

        // Update the entity and client sets attached to each condition.
        for condition_id in condition_ids.iter()
        {
            let Some((condition, entities, clients)) = self.conditions.get_mut(condition_id)
            else { tracing::error!("missing condition on update client visibility"); continue; };

            // Evaluate client visibility for this condition.
            let visibility = condition.evaluate(|a| client_attributes.contains(&a));

            // Save the client's visibility of this condition.
            match visibility
            {
                true  => { clients.insert(client_id); }
                false => { clients.remove(&client_id); }
            }

            // Set visibility for entities attached to this condition.
            // - Ignore disconnected clients.
            tracing::trace!(?client_id, ?entities, "visibility {visibility}");
            let Some(ref mut visibility_settings) = visibility_settings else { continue; };

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
        client_cache     : &mut ClientCache,
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
        tracing::trace!(?entity, ?condition_id, "removed condition from entity");

        // Access conditions map.
        let Some((_, entities, clients)) = self.conditions.get_mut(&condition_id)
        else { tracing::error!("missing condition on remove entity"); return true; };

        // Remove entity from tracked set for this condition.
        if !entities.remove(&entity)
        { tracing::error!("missing entity on remove entity"); }

        // Update visibility of this entity for clients that can see this condition.
        for client_id in clients.iter()
        {
            tracing::trace!(?client_id, ?entity, "visibility false");
            let Some(client) = client_cache.get_client_mut(*client_id) else { continue; };
            client.visibility_mut().set_visibility(entity, false);
        }

        // Cleanup
        if entities.len() == 0
        {
            // remove condition
            let (condition, mut entities, mut clients) = self.conditions.remove(&condition_id).unwrap();

            // remove condition from attributes map
            for attribute_id in condition.iter_attributes()
            {
                let Some(condition_ids) = self.attributes.get_mut(&attribute_id)
                else { tracing::error!("missing attribute on remove entity cleanup"); continue; };

                if !condition_ids.remove(&condition_id)
                { tracing::error!("missing condition on remove entity cleanup"); continue; }

                // Cleanup
                if condition_ids.len() == 0
                {
                    let mut condition_ids = self.attributes.remove(&attribute_id).unwrap();
                    condition_ids.clear();
                    self.condition_ids_buffer.push(condition_ids);
                }
            }

            // save buffers
            entities.clear();
            clients.clear();
            self.entities_buffer.push(entities);
            self.client_ids_buffer.push(clients);
        }

        true
    }
}

impl Default for VisibilityCache
{
    fn default() -> Self
    {
        Self{
            attributes: HashMap::default(),
            entities: EntityHashMap::default(),
            conditions: HashMap::default(),
            clients: HashMap::default(),
            condition_ids_buffer: Vec::default(),
            entities_buffer: Vec::default(),
            client_ids_buffer: Vec::default(),
            attribute_ids_buffer: Vec::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
