//local shortcuts
use crate::*;

//third-party shortcuts
use bevy_replicon::prelude::ClientId;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Builtin [`VisibilityAttribute`] added to all clients by default.
///
/// Add this attribute to your entity visibility conditions if you want them to be globally visibile.
#[derive(VisibilityAttribute, Default, PartialEq, Eq, Copy, Clone, Debug)]
pub struct Global;

//-------------------------------------------------------------------------------------------------------------------

/// Builtin [`VisibilityAttribute`] added to all clients by default.
///
/// Add this attribute to your entity visibility conditions if you want them to visibile to a specific client.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Client(pub ClientId);

impl From<u64> for Client
{
    fn from(id: u64) -> Self
    {
        Client(ClientId::new(id))
    }
}

impl VisibilityAttribute for Client
{
    fn inner_attribute_id(&self) -> u64
    {
        self.0.get()
    }
}

//-------------------------------------------------------------------------------------------------------------------
