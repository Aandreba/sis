use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemStruct, Fields};

mod builder;

#[proc_macro_attribute]
pub fn self_referencing (attrs: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    
    return quote! {
        #(#attrs)*
    }.into()
}