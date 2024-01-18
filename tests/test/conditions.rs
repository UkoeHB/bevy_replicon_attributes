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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), None);
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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Manual(1).attribute_id()));
    assert_eq!(iter.next(), None);
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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), None);
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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), Some(Manual(0).attribute_id()));
    assert_eq!(iter.next(), None);
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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), Some(Manual(0).attribute_id()));
    assert_eq!(iter.next(), None);
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

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), Some(Manual(0).attribute_id()));
    assert_eq!(iter.next(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn condition_ids()
{
    let attribute1 = VisibilityCondition::new(attr(Test));
    let attribute2 = VisibilityCondition::new(attr(Dummy));
    let attribute3 = VisibilityCondition::new(attr(Manual(1)));
    let attribute4 = VisibilityCondition::new(attr(Manual(2)));

    assert_eq!(attribute1.condition_id(), attribute1.condition_id());
    assert_ne!(attribute1.condition_id(), attribute2.condition_id());
    assert_ne!(attribute1.condition_id(), attribute3.condition_id());

    assert_eq!(attribute3.condition_id(), attribute3.condition_id());
    assert_ne!(attribute3.condition_id(), attribute4.condition_id());


    let not1 = VisibilityCondition::new(not(attr(Test)));
    let not2 = VisibilityCondition::new(not(attr(Dummy)));
    let not3 = VisibilityCondition::new(not(attr(Manual(1))));
    let not4 = VisibilityCondition::new(not(attr(Manual(2))));

    assert_eq!(not1.condition_id(), not1.condition_id());
    assert_ne!(not1.condition_id(), not2.condition_id());
    assert_ne!(not1.condition_id(), not3.condition_id());
    assert_ne!(not1.condition_id(), attribute1.condition_id());

    assert_eq!(not3.condition_id(), not3.condition_id());
    assert_ne!(not3.condition_id(), not4.condition_id());
    assert_ne!(not3.condition_id(), attribute3.condition_id());


    let and1 = VisibilityCondition::new(and(attr(Test), attr(Test)));
    let and2 = VisibilityCondition::new(and(attr(Test), attr(Dummy)));
    let and3 = VisibilityCondition::new(and(attr(Test), attr(Manual(1))));
    let and4 = VisibilityCondition::new(and(attr(Test), attr(Manual(2))));

    assert_eq!(and1.condition_id(), and1.condition_id());
    assert_ne!(and1.condition_id(), and2.condition_id());
    assert_ne!(and1.condition_id(), and3.condition_id());
    assert_ne!(and1.condition_id(), attribute1.condition_id());
    assert_ne!(and1.condition_id(), not1.condition_id());

    assert_eq!(and3.condition_id(), and3.condition_id());
    assert_ne!(and3.condition_id(), and4.condition_id());
    assert_ne!(and3.condition_id(), attribute3.condition_id());
    assert_ne!(and3.condition_id(), not3.condition_id());


    let or1 = VisibilityCondition::new(or(attr(Test), attr(Test)));
    let or2 = VisibilityCondition::new(or(attr(Test), attr(Dummy)));
    let or3 = VisibilityCondition::new(or(attr(Test), attr(Manual(1))));
    let or4 = VisibilityCondition::new(or(attr(Test), attr(Manual(2))));

    assert_eq!(or1.condition_id(), or1.condition_id());
    assert_ne!(or1.condition_id(), or2.condition_id());
    assert_ne!(or1.condition_id(), or3.condition_id());
    assert_ne!(or1.condition_id(), attribute1.condition_id());
    assert_ne!(or1.condition_id(), not1.condition_id());
    assert_ne!(or1.condition_id(), and1.condition_id());

    assert_eq!(or3.condition_id(), or3.condition_id());
    assert_ne!(or3.condition_id(), or4.condition_id());
    assert_ne!(or3.condition_id(), attribute3.condition_id());
    assert_ne!(or3.condition_id(), not3.condition_id());
    assert_ne!(or3.condition_id(), and3.condition_id());
}

//-------------------------------------------------------------------------------------------------------------------
