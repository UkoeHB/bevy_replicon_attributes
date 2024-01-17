//local shortcuts
use bevy_replicon_attributes::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(VisibilityAttribute, Default, Eq, PartialEq)]
struct Test;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn basic()
{
    let visible_to = VisibleTo::new(Test);
    let visible_to = VisibleTo::new(not(Test));
    let visible_to = VisibleTo::new(and(Test, Test));
    let visible_to = VisibleTo::new(or(Test, Test));
    let visible_to = VisibleTo::new(or(not(Test), and(Test, not(Test))));
}

//-------------------------------------------------------------------------------------------------------------------
