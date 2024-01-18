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
    //let _ = visible_to!(!Test);  //disabled temporarily until +/| also work
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
    let condition = visible_to!(Test);
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
    let condition = visible_to!(Manual(1));
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
    let condition = visible_to!(not(Test));
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
    let condition = visible_to!(and(Test, Manual(0)));
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
    let condition = visible_to!(or(Test, Manual(0)));
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
    let condition = visible_to!(or(not(Test), and(Test, Manual(0))));
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
fn condition_composition()
{
    let a = visible_to!(Test);
    let b = visible_to!(not(Dummy));
    let c = visible_to!(or(Manual(0), Manual(1)));
    let combo = visible_to!(and(a, and(b, c)));  //Test + !Dummy + (Manual(0) | Manual(1))

    assert!(!combo.evaluate(|a| a == Test.attribute_id()));
    assert!(!combo.evaluate(|a| a == Manual(0).attribute_id()));
    assert!(!combo.evaluate(|a| a == Dummy.attribute_id()));
    assert!(combo.evaluate(|a| a == Test.attribute_id() || a == Manual(0).attribute_id()));

    let mut iter = combo.iter_attributes();
    assert_eq!(iter.next(), Some(Test.attribute_id()));
    assert_eq!(iter.next(), Some(Dummy.attribute_id()));
    assert_eq!(iter.next(), Some(Manual(0).attribute_id()));
    assert_eq!(iter.next(), Some(Manual(1).attribute_id()));
    assert_eq!(iter.next(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn condition_ids()
{
    // ATTRIBUTE
    let attribute1 = visible_to!(Test);
    let attribute2 = visible_to!(Dummy);
    let attribute3 = visible_to!(Manual(1));
    let attribute4 = visible_to!(Manual(2));

    assert_eq!(attribute1.condition_id(), attribute1.condition_id());
    assert_ne!(attribute1.condition_id(), attribute2.condition_id());
    assert_ne!(attribute1.condition_id(), attribute3.condition_id());

    assert_eq!(attribute3.condition_id(), attribute3.condition_id());
    assert_ne!(attribute3.condition_id(), attribute4.condition_id());

    // NOT
    let not1 = visible_to!(not(Test));
    let not2 = visible_to!(not(Dummy));
    let not3 = visible_to!(not(Manual(1)));
    let not4 = visible_to!(not(Manual(2)));

    assert_eq!(not1.condition_id(), not1.condition_id());
    assert_ne!(not1.condition_id(), not2.condition_id());
    assert_ne!(not1.condition_id(), not3.condition_id());
    assert_ne!(not1.condition_id(), attribute1.condition_id());

    assert_eq!(not3.condition_id(), not3.condition_id());
    assert_ne!(not3.condition_id(), not4.condition_id());
    assert_ne!(not3.condition_id(), attribute3.condition_id());

    // AND
    let and1 = visible_to!(and(Test, Test));
    let and2 = visible_to!(and(Test, Dummy));
    let and3 = visible_to!(and(Test, Manual(1)));
    let and4 = visible_to!(and(Test, Manual(2)));

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
    let or1 = visible_to!(or(Test, Test));
    let or2 = visible_to!(or(Test, Dummy));
    let or3 = visible_to!(or(Test, Manual(1)));
    let or4 = visible_to!(or(Test, Manual(2)));

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
