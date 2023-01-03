use std::{marker::PhantomPinned, mem::MaybeUninit, pin::Pin};

#[macro_export]
macro_rules! pin_mut {
    ($($x:ident),* $(,)?) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $x = unsafe {
            ::core::pin::Pin::new_unchecked(&mut $x)
        };
    )* }
}

struct Test<'this> {
    value: i32,
    rf: MaybeUninit<&'this i32>,
    _pin: PhantomPinned
}

impl<'this> Test<'this> {
    #[inline]
    pub unsafe fn new_uninit (value: i32) -> Self {
        return Self {
            value,
            rf: MaybeUninit::uninit(),
            _pin: PhantomPinned
        }
    }

    #[inline]
    pub fn initialize (self: Pin<&'this mut Self>) {
        unsafe {
            let Self { value, rf, _pin }: &'this mut Self = Pin::into_inner_unchecked(self);
            rf.write(value);
        }
    }
}

#[inline]
pub fn test () {
    let mut v = unsafe { Test::new_uninit(3) };
    let v = unsafe { Pin::new_unchecked(&mut v) };
    v.initialize();
}