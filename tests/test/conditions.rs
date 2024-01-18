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
fn test_macro()
{
    let _ = visible_to!(Test);
    let _ = visible_to!(!Test);
    //let _ = visible_to!(Test + Test);
    //let _ = visible_to!(Test | Test);
    //let _ = visible_to!(!Test | !Test);
    //let _ = visible_to!(!(Test + Test) | (Test | Test));
    let _ = visible_to!(and(Test, Test));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn derive_attr_check()
{
    let condition = VisibilityCondition::new(Test);
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
    let condition = VisibilityCondition::new(Manual(1));
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
    let condition = VisibilityCondition::new(not(Test));
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
    let condition = VisibilityCondition::new(and(Test, Manual(0)));
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
    let condition = VisibilityCondition::new(or(Test, Manual(0)));
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
    let condition = VisibilityCondition::new(or(not(Test), and(Test, Manual(0))));
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
    // ATTRIBUTE
    let attribute1 = VisibilityCondition::new(Test);
    let attribute2 = VisibilityCondition::new(Dummy);
    let attribute3 = VisibilityCondition::new(Manual(1));
    let attribute4 = VisibilityCondition::new(Manual(2));

    assert_eq!(attribute1.condition_id(), attribute1.condition_id());
    assert_ne!(attribute1.condition_id(), attribute2.condition_id());
    assert_ne!(attribute1.condition_id(), attribute3.condition_id());

    assert_eq!(attribute3.condition_id(), attribute3.condition_id());
    assert_ne!(attribute3.condition_id(), attribute4.condition_id());

    // NOT
    let not1 = VisibilityCondition::new(not(Test));
    let not2 = VisibilityCondition::new(not(Dummy));
    let not3 = VisibilityCondition::new(not(Manual(1)));
    let not4 = VisibilityCondition::new(not(Manual(2)));

    assert_eq!(not1.condition_id(), not1.condition_id());
    assert_ne!(not1.condition_id(), not2.condition_id());
    assert_ne!(not1.condition_id(), not3.condition_id());
    assert_ne!(not1.condition_id(), attribute1.condition_id());

    assert_eq!(not3.condition_id(), not3.condition_id());
    assert_ne!(not3.condition_id(), not4.condition_id());
    assert_ne!(not3.condition_id(), attribute3.condition_id());

    // AND
    let and1 = VisibilityCondition::new(and(Test, Test));
    let and2 = VisibilityCondition::new(and(Test, Dummy));
    let and3 = VisibilityCondition::new(and(Test, Manual(1)));
    let and4 = VisibilityCondition::new(and(Test, Manual(2)));

    assert_eq!(and1.condition_id(), and1.condition_id());
    assert_ne!(and1.condition_id(), and2.condition_id());
    assert_ne!(and1.condition_id(), and3.condition_id());
    assert_ne!(and1.condition_id(), attribute1.condition_id());
    assert_ne!(and1.condition_id(), not1.condition_id());

    assert_eq!(and3.condition_id(), and3.condition_id());
    assert_ne!(and3.condition_id(), and4.condition_id());
    assert_ne!(and3.condition_id(), attribute3.condition_id());
    assert_ne!(and3.condition_id(), not3.condition_id());

    // OR
    let or1 = VisibilityCondition::new(or(Test, Test));
    let or2 = VisibilityCondition::new(or(Test, Dummy));
    let or3 = VisibilityCondition::new(or(Test, Manual(1)));
    let or4 = VisibilityCondition::new(or(Test, Manual(2)));

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
