//local shortcuts
use bevy_replicon_attributes::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(VisibilityAttribute, Default, Eq, PartialEq)]
struct Dummy;

#[derive(VisibilityAttribute, Default, Eq, PartialEq)]
struct Test;

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
fn derive_attr_check()
{
    let condition = VisibilityCondition::new(attr(Test));
    assert!(condition.evaluate(|a| a == Test.attribute_id()));
    assert!(!condition.evaluate(|a| a != Test.attribute_id()));
    assert!(!condition.evaluate(|a| a == Dummy.attribute_id()));
    assert!(condition.evaluate(|a| a != Dummy.attribute_id()));
    assert!(!condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(condition.evaluate(|a| a != Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a == Manual(1).attribute_id()));
    assert!(condition.evaluate(|a| a != Manual(1).attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn manual_attr_check()
{
    let condition = VisibilityCondition::new(attr(Manual(1)));
    assert!(condition.evaluate(|a| a == Manual(1).attribute_id()));
    assert!(!condition.evaluate(|a| a != Manual(1).attribute_id()));
    assert!(!condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(condition.evaluate(|a| a != Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a == Test.attribute_id()));
    assert!(condition.evaluate(|a| a != Test.attribute_id()));
    assert!(!condition.evaluate(|a| a == Dummy.attribute_id()));
    assert!(condition.evaluate(|a| a != Dummy.attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn not_check()
{
    let condition = VisibilityCondition::new(not(attr(Test)));
    assert!(!condition.evaluate(|a| a == Test.attribute_id()));
    assert!(condition.evaluate(|a| a != Test.attribute_id()));
    assert!(condition.evaluate(|a| a == Dummy.attribute_id()));
    assert!(!condition.evaluate(|a| a != Dummy.attribute_id()));
    assert!(condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a != Manual(0).attribute_id()));
    assert!(condition.evaluate(|a| a == Manual(1).attribute_id()));
    assert!(!condition.evaluate(|a| a != Manual(1).attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn and_check()
{
    let condition = VisibilityCondition::new(and(attr(Test), attr(Manual(0))));
    assert!(!condition.evaluate(|a| a == Test.attribute_id()));
    assert!(!condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(condition.evaluate(|a| a == Test.attribute_id() || a == Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a == Test.attribute_id() && a == Manual(0).attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn or_check()
{
    let condition = VisibilityCondition::new(or(attr(Test), attr(Manual(0))));
    assert!(condition.evaluate(|a| a == Test.attribute_id()));
    assert!(condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a == Dummy.attribute_id()));
    assert!(!condition.evaluate(|a| a == Manual(1).attribute_id()));
    assert!(condition.evaluate(|a| a == Test.attribute_id() || a == Manual(0).attribute_id()));
    assert!(!condition.evaluate(|a| a == Test.attribute_id() && a == Manual(0).attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn combo_check()
{
    let condition = VisibilityCondition::new(or(not(attr(Test)), and(attr(Test), attr(Manual(0)))));
    assert!(!condition.evaluate(|a| a == Test.attribute_id()));
    assert!(condition.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(condition.evaluate(|a| a == Dummy.attribute_id()));
    assert!(condition.evaluate(|a| a == Test.attribute_id() || a == Manual(0).attribute_id()));
}

//-------------------------------------------------------------------------------------------------------------------
