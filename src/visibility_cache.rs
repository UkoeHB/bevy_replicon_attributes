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
    /// The id for `VisibilityConditionId::empty()`.
    empty_condition_id: VisibilityConditionId,

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
    /// Makes a new cache.
    pub(crate) fn new() -> Self
    {
        Self{
            empty_condition_id: VisibilityCondition::empty().condition_id(),
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

    /// Resets a client client in the cache.
    ///
    /// If the client already has attributes regisistered, they are cleared.
    ///
    /// The client will evaluate its visibility against the 'empty' visibility condition.
    pub(crate) fn reset_client(
        &mut self,
        client_cache : &mut ClientCache,
        client_id    : ClientId,
    ){
        tracing::debug!(?client_id, "resetting client");

        // Remove the client
        self.remove_client(client_id);

        // Add a new entry for the client that was just removed.
        self.clients.insert(client_id, self.attribute_ids_buffer.pop().unwrap_or_default());

        // Update the client set attached to the empty visibility condition.
        let Some((condition, entities, ref mut clients)) = self.conditions.get_mut(&self.empty_condition_id) else { return; };

        // Save the client's visibility of this condition.
        clients.insert(client_id);

        // Set visibility for entities attached to the empty visibility condition.
        let Some(visibility_settings) = client_cache.get_client_mut(client_id).map(|c| c.visibility_mut())
        else { tracing::error!(?client_id, "resetting client is missing from client cache"); return; };

        for entity in entities.iter()
        {
            tracing::trace!(?client_id, ?entity, ?condition, "visibility <true>");
            visibility_settings.set_visibility(*entity, true);
        }
    }

    /// Removes a client.
    pub(crate) fn remove_client(&mut self, client_id: ClientId)
    {
        tracing::debug!(?client_id, "removing client");

        // Remove client entry
        let Some(mut attribute_ids) = self.clients.remove(&client_id) else { return; };

        // Find conditions monitored by this client.
        for attribute_id in attribute_ids.iter()
        {
            // We do not log an error on failure because the client may have an attribute with no corresponding conditions.
            let Some(condition_ids) = self.attributes.get(attribute_id) else { continue; };

            for condition_id in condition_ids.iter()
            {
                let Some((_, _, clients)) = self.conditions.get_mut(condition_id)
                else { tracing::error!(?client_id, "condition missing on remove client"); continue; };

                // Clean up the client.
                // - We do not log an error on failure because this client may not have visibility of this condition.
                clients.remove(&client_id);
            }
        }

        // Cache the attributes buffer for a future client.
        attribute_ids.clear();
        self.attribute_ids_buffer.push(attribute_ids);
    }

    /// Repairs a client by refreshing visibility of all entities in the [`ClientCache`].
    pub(crate) fn repair_client(&mut self, client_cache: &mut ClientCache, client_id: ClientId)
    {
        tracing::debug!(?client_id, "repairing client");

        // Access client attributes.
        let client_attributes = self.clients
            .entry(client_id)
            .or_insert_with(|| self.attribute_ids_buffer.pop().unwrap_or_default());

        // Get client visibility settings.
        let Some(visibility_settings) = client_cache.get_client_mut(client_id).map(|c| c.visibility_mut())
        else { tracing::error!(?client_id, "repairing client is missing from client cache"); return; };

        // Prep evaluator
        let self_conditions = &mut self.conditions;
        let mut evaluator = |condition_id: &VisibilityConditionId| -> bool
        {
            let Some((condition, entities, clients)) = self_conditions.get_mut(condition_id) else { return false; };

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
            tracing::trace!(?client_id, ?entities, ?condition, "visibility <{visibility}>");

            for entity in entities.iter()
            {
                visibility_settings.set_visibility(*entity, visibility);
            }

            true
        };

        // Evaluate the 'empty' visibility condition
        evaluator(&self.empty_condition_id);

        // Iterate all client attributes
        for client_attribute in client_attributes.iter()
        {
            // Access conditions associated with this attribute.
            let Some(condition_ids) = self.attributes.get(&client_attribute) else { continue; };

            // Update the client sets attached to each condition.
            for condition_id in condition_ids.iter()
            {
                if !evaluator(condition_id)
                { tracing::error!(?client_id, ?condition_id, "missing condition on repair client visibility"); }
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
        { tracing::debug!(?entity, ?condition, "ignoring attempt to add an entity condition that already exists"); return; }

        // Update entity map.
        if self.entities.insert(entity, condition_id).is_some()
        { tracing::error!(?entity, ?condition, "entity unexpectedly had condition on insert"); return; }
        tracing::trace!(?entity, ?condition, "added condition to entity");

        // Access conditions map.
        let entry = self.conditions.entry(condition_id);

        // Update attributes map if this is a new condition.
        let is_new_condition = if let Entry::Vacant(_) = &entry { true } else { false };

        if is_new_condition
        {
            for attribute_id in condition.iter_attributes()
            {
                if !self.attributes
                    .entry(attribute_id)
                    .or_insert_with(|| self.condition_ids_buffer.pop().unwrap_or_default())
                    .insert(condition_id)
                { tracing::error!(?attribute_id, ?condition, "found condition in attributes map without conditions entry"); }
            }
        }

        // Update conditions map.
        let (_, entities, ref mut clients) = entry
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
        { tracing::error!(?entity, ?condition, "entity unexpectedly in tracked entities for condition"); }

        // Update clients
        match is_new_condition
        {
            // Establish initial visibility for the new condition.
            true =>
            {
                for (client_id, attributes) in self.clients.iter()
                {
                    if !condition.evaluate(|a| attributes.contains(&a)) { continue }

                    clients.insert(*client_id);

                    tracing::trace!(?client_id, ?entity, ?condition, "visibility <true> new condition");
                    let Some(client) = client_cache.get_client_mut(*client_id) else { continue; };
                    client.visibility_mut().set_visibility(entity, true);
                }
            }
            // Update visibility of this entity for clients that can see this condition.
            // - Note that it's possible we are setting client visibilities back to `true` that were set to `false`
            //   when cleaning up this entity's old condition. We assume performance is about equivalent between
            //   this brute-force approach and trying to only modify clients that don't have visibility of both conditions.
            false =>
            {
                for client_id in clients.iter()
                {
                    tracing::trace!(?client_id, ?entity, ?condition, "visibility <true>");
                    let Some(client) = client_cache.get_client_mut(*client_id) else { continue; };
                    client.visibility_mut().set_visibility(entity, true);
                }
            }
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

    /// Evaluates a visibility condition againt all clients and returns an iterator of clients that evaluate true.
    pub(crate) fn iter_client_visibility<'s, 'a: 's>(
        &'s self,
        condition: &'a VisibilityCondition
    ) -> impl Iterator<Item = ClientId> + '_
    {
        self.clients
            .iter()
            .filter_map(
                |(id, attrs)|
                {
                    match condition.evaluate(|a| attrs.contains(&a))
                    {
                        true  => Some(*id),
                        false => None,
                    }
                }
            )
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
            else { tracing::error!(?client_id, "missing condition on update client visibility"); continue; };

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
            tracing::trace!(?client_id, ?entities, ?condition, "visibility {visibility}");
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
        let Some((condition, entities, clients)) = self.conditions.get_mut(&condition_id)
        else { tracing::error!(?entity, ?condition_id, "missing condition on remove entity"); return true; };

        // Remove entity from tracked set for this condition.
        if !entities.remove(&entity)
        { tracing::error!(?entity, "missing entity on remove entity"); }

        // Update visibility of this entity for clients that can see this condition.
        for client_id in clients.iter()
        {
            tracing::trace!(?client_id, ?entity, ?condition, "visibility false");
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
                else { tracing::error!(?entity, ?attribute_id, "missing attribute on remove entity cleanup"); continue; };

                if !condition_ids.remove(&condition_id)
                { tracing::error!(?entity, ?condition_id, "missing condition on remove entity cleanup"); continue; }

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

//-------------------------------------------------------------------------------------------------------------------
