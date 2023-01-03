#![feature(type_name_of_val)]

use std::pin::Pin;
use sis::self_referencing;

#[self_referencing]
#[derive(Debug)]
struct Test {
    str1: String,
    str2: String,
    #[borrows(str1)]
    left: &'this str,
    #[borrows(mut str2)]
    right: &'this mut str,
}

#[test]
fn test () {
    new_test! {
        { "Hello".to_string(), "World".to_string() },
        { |x| &x.get_ref()[..2], |x| &mut x.get_mut()[2..] },
        walue
    };

    new_test! {
        { "Alex".to_string(), "Andreba".to_string() },
        { |x| &x.get_ref()[..2], |x| &mut x.get_mut()[2..] },
        box boxed
    }

    let walue: Pin<&mut Test> = walue;
    let boxed: Pin<Box<Test>> = boxed;

    let alpha = walue.left();
    let beta = walue.right();

    println!("{alpha:?}, {beta:?}");
}