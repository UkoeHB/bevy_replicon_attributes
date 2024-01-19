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
fn test_macro()
{
    let _ = vis!(Test);
    //let _ = vis!(!Test);  //disabled temporarily until +/| also work
    //let _ = vis!(Test + Test);
    //let _ = vis!(Test | Test);
    //let _ = vis!(!Test | !Test);
    //let _ = vis!(!(Test + Test) | (Test | Test));
    let _ = vis!(and(Test, Test));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn empty_check()
{
    let condition = vis!();
    assert!(condition.evaluate(|_| false));
    assert!(condition.evaluate(|_| true));

    let mut iter = condition.iter_attributes();
    assert_eq!(iter.next(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn derive_attr_check()
{
    let condition = vis!(Test);
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
    let condition = vis!(Manual(1));
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
    let condition = vis!(not(Test));
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
    let condition = vis!(and(Test, Manual(0)));
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
    let condition = vis!(or(Test, Manual(0)));
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
    let condition = vis!(or(not(Test), and(Test, Manual(0))));
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
    let a = vis!(Test);
    let b = vis!(not(Dummy));
    let c = vis!(or(Manual(0), Manual(1)));
    let combo = vis!(and(a, and(b, c)));  //Test + !Dummy + (Manual(0) | Manual(1))

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
    let attribute1 = vis!(Test);
    let attribute2 = vis!(Dummy);
    let attribute3 = vis!(Manual(1));
    let attribute4 = vis!(Manual(2));

    assert_eq!(attribute1.condition_id(), attribute1.condition_id());
    assert_ne!(attribute1.condition_id(), attribute2.condition_id());
    assert_ne!(attribute1.condition_id(), attribute3.condition_id());

    assert_eq!(attribute3.condition_id(), attribute3.condition_id());
    assert_ne!(attribute3.condition_id(), attribute4.condition_id());

    // NOT
    let not1 = vis!(not(Test));
    let not2 = vis!(not(Dummy));
    let not3 = vis!(not(Manual(1)));
    let not4 = vis!(not(Manual(2)));

    assert_eq!(not1.condition_id(), not1.condition_id());
    assert_ne!(not1.condition_id(), not2.condition_id());
    assert_ne!(not1.condition_id(), not3.condition_id());
    assert_ne!(not1.condition_id(), attribute1.condition_id());

    assert_eq!(not3.condition_id(), not3.condition_id());
    assert_ne!(not3.condition_id(), not4.condition_id());
    assert_ne!(not3.condition_id(), attribute3.condition_id());

    // AND
    let and1 = vis!(and(Test, Test));
    let and2 = vis!(and(Test, Dummy));
    let and3 = vis!(and(Test, Manual(1)));
    let and4 = vis!(and(Test, Manual(2)));

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
    let or1 = vis!(or(Test, Test));
    let or2 = vis!(or(Test, Dummy));
    let or3 = vis!(or(Test, Manual(1)));
    let or4 = vis!(or(Test, Manual(2)));

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

    // COMBO
    let combo1 = vis!(or(Test, and(not(Test), Dummy)));
    let combo2 = vis!(and(Test, or(Manual(0), Dummy)));

    assert_eq!(combo1.condition_id(), combo1.condition_id());
    assert_ne!(combo1.condition_id(), combo2.condition_id());
    assert_ne!(combo1.condition_id(), attribute1.condition_id());
    assert_ne!(combo1.condition_id(), not1.condition_id());
    assert_ne!(combo1.condition_id(), and1.condition_id());
    assert_ne!(combo1.condition_id(), or1.condition_id());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn visibility_extension()
{
    let mut a = vis!(or(A, B));
    a.and(and(not(C), D));
    assert_eq!(a, vis!(and(or(A, B), and(not(C), D))));

    let mut a = vis!(not(A));
    a.or(or(B, C));
    assert_eq!(a, vis!(or(not(A), or(B, C))));
}

//-------------------------------------------------------------------------------------------------------------------
