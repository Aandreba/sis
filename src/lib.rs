#![no_std]
#![feature(unboxed_closures, fn_traits, tuple_trait)]

extern crate sis_proc;
pub use sis_proc::*;

use core::{future::Future, marker::Tuple};

#[doc(hidden)]
pub extern crate core;

pub trait FutureGenerator<Args: Tuple>: FnOnce<Args> {
    type Future: Future<Output = <Self as FutureGenerator<Args>>::Output>;
    type Output;

    fn call (self, args: Args) -> Self::Future;
}

impl<Args: Tuple, F: FnOnce<Args>> FutureGenerator<Args> for F where F::Output: Future {
    type Future = F::Output;
    type Output = <F::Output as Future>::Output;

    #[inline]
    fn call (self, args: Args) -> Self::Future {
        self.call_once(args)
    }
}