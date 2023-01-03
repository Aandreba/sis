use attrs::SisAttrs;
use builder::self_referencing_impl;
use syn::{parse_macro_input, ItemStruct};

mod attrs;
mod builder;

#[proc_macro_attribute]
pub fn self_referencing (attrs: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let SisAttrs { attrs } = parse_macro_input!(attrs as SisAttrs);
    let item = parse_macro_input!(item as ItemStruct);
    return self_referencing_impl(&attrs, item).into()
}

pub(crate) fn to_snake_case (camel_case: &str) -> String {
    let mut result = String::with_capacity(camel_case.len());
    let mut chars = camel_case.chars();

    if let Some(first) = chars.next() {
        result.extend(first.to_lowercase());
    }

    for c in chars {
        if c.is_uppercase() {
            result.push('_');
            result.extend(c.to_lowercase())
        } else {
            result.push(c);
        }
    }

    return result
}