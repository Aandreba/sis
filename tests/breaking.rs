use std::marker::PhantomData;
use sis::self_referencing;

#[self_referencing]
#[derive(Debug)]
struct GameInner {
    _path: &'this str,
    common: &'this str,
    religions: String,
    countries: String,
    #[borrows(religions, common)]
    cultures: &'this String,
    #[borrows(mut countries, cultures)]
    _country_init: PhantomData<&'this mut ()>
}