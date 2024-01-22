//local shortcuts
use bevy_replicon_attributes::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(VisibilityAttribute, Default, PartialEq)]
struct Dummy;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct Test;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct A;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct B;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct C;

#[derive(VisibilityAttribute, Default, PartialEq)]
struct D;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct Manual(usize);

impl VisibilityAttribute for Manual
{
    fn inner_attribute_id(&self) -> u64
    {
        self.0 as u64
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[test]
fn test()
{

}

//-------------------------------------------------------------------------------------------------------------------

// event sent to empty is seen by all
// event sent to condition is seen by matching clients
// event sent with empty vis before client connects is not seen
// multiple events sent in the same tick are all seen by clients
