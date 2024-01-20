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

struct Manual2(usize);

impl VisibilityAttribute for Manual2
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

    assert_eq!(vis!(empty()), vis!());
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

    assert_eq!(vis!(not(empty())), vis!());
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

    assert_eq!(vis!(and(empty(), A)), vis!(A));
    assert_eq!(vis!(and(A, empty())), vis!(A));
    assert_eq!(vis!(and(empty(), empty())), vis!());
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

    assert_eq!(vis!(or(empty(), A)), vis!(A));
    assert_eq!(vis!(or(A, empty())), vis!(A));
    assert_eq!(vis!(or(empty(), empty())), vis!());
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
fn all_check()
{
    assert_eq!(vis!(all!()), vis!(empty()));
    assert_eq!(vis!(all!(A)), vis!(A));
    assert_eq!(vis!(all!(A, B, C)), vis!(and(A, and(B, C))));
    assert_eq!(vis!(all!(A, and(B, C))), vis!(and(A, and(B, C))));
    assert_eq!(vis!(all!(not(A), B)), vis!(and(not(A), B)));
    assert_eq!(vis!(all!(A, not(B))), vis!(and(A, not(B))));
    assert_eq!(vis!(all!(A, all!(B, C))), vis!(and(A, and(B, C))));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn any_check()
{
    assert_eq!(vis!(any!()), vis!(empty()));
    assert_eq!(vis!(any!(A)), vis!(A));
    assert_eq!(vis!(any!(A, B, C)), vis!(or(A, or(B, C))));
    assert_eq!(vis!(any!(A, or(B, C))), vis!(or(A, or(B, C))));
    assert_eq!(vis!(any!(not(A), B)), vis!(or(not(A), B)));
    assert_eq!(vis!(any!(A, not(B))), vis!(or(A, not(B))));
    assert_eq!(vis!(any!(A, any!(B, C))), vis!(or(A, or(B, C))));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn none_check()
{
    assert_eq!(vis!(none!()), vis!(empty()));
    assert_eq!(vis!(none!(A)), vis!(not(A)));
    assert_eq!(vis!(none!(A, B, C)), vis!(not(or(A, or(B, C)))));
    assert_eq!(vis!(none!(A, or(B, C))), vis!(not(or(A, or(B, C)))));
    assert_eq!(vis!(none!(not(A), B)), vis!(not(or(not(A), B))));
    assert_eq!(vis!(none!(A, not(B))), vis!(not(or(A, not(B)))));
    assert_eq!(vis!(none!(A, none!(B, C))), vis!(not(or(A, not(or(B, C))))));
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
fn consolidation()
{
    assert_eq!(vis!(and(A, not(empty()))), vis!(A));
    assert_eq!(vis!(and(not(empty()), A)), vis!(A));
    assert_eq!(vis!(or(A, not(empty()))), vis!(A));
    assert_eq!(vis!(or(not(empty()), A)), vis!(A));

    assert_eq!(vis!(and(and(A, empty()), empty())), vis!(A));
    assert_eq!(vis!(and(empty(), and(empty(), A))), vis!(A));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn extension()
{
    let mut a = vis!(or(A, B));
    a.and(and(not(C), D));
    assert_eq!(a, vis!(and(or(A, B), and(not(C), D))));

    let mut a = vis!(not(A));
    a.or(or(B, C));
    assert_eq!(a, vis!(or(not(A), or(B, C))));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn replacement()
{
    let mut a = vis!(and(A, B));
    a.replace(B, not(B));
    assert_eq!(a, vis!(and(A, not(B))));

    a.replace(and(A, not(B)), C);
    assert_eq!(a, vis!(C));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn replacement_by_type()
{
    let mut a = vis!(and(Manual(0), not(Manual(1))));
    a.replace_type::<Manual>(B);
    assert_eq!(a, vis!(and(B, not(B))));

    let mut m = vis!(Manual(20));
    m.replace_type::<Manual>(Manual(22));
    assert_eq!(m, vis!(Manual(22)));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn removal()
{
    let mut a = vis!(and(A, B));
    a.remove(B);
    assert_eq!(a, vis!(A));

    let mut b = vis!(or(A, not(B)));
    b.remove(or(A, not(B)));
    assert_eq!(b, vis!());
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn removal_by_type()
{
    let mut a = vis!(and(Manual(0), not(Manual(1))));
    a.remove_type::<Manual>();
    assert_eq!(a, vis!());

    let mut b = vis!(or(Manual(1), Manual2(2)));
    b.remove_type::<Manual>();
    assert_eq!(b, vis!(Manual2(2)));
}

//-------------------------------------------------------------------------------------------------------------------
