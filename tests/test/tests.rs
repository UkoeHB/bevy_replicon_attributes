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
    let visible_to = VisibleTo::new(attr(Test));
    let visible_to = VisibleTo::new(not(attr(Test)));
    let visible_to = VisibleTo::new(and(attr(Test), attr(Test)));
    let visible_to = VisibleTo::new(or(attr(Test), attr(Test)));
    let visible_to = VisibleTo::new(or(not(attr(Test)), and(attr(Test), not(attr(Test)))));
}

//-------------------------------------------------------------------------------------------------------------------
