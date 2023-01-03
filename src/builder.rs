use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{FieldsNamed, Fields, ItemStruct, FieldsUnnamed, punctuated::Punctuated, Token, Field, parse_quote};

#[inline]
fn create_builder (ItemStruct { attrs, vis, struct_token, ident, generics, fields, semi_token }: &mut ItemStruct) -> TokenStream {
    let builder_ident = format_ident!("{ident}Builder"); 
    let fields = match fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        _ => todo!()
    };

    return quote! {
        #(#attrs)*
        #vis #struct_token #builder_ident #generics {

        }
    }
}

fn builder_fields (fields: &mut Punctuated<Field, Token![,]>) -> Vec<TokenStream> {
    let mut output = Vec::with_capacity(fields.len());
    for (i, field) in fields.iter_mut().enumerate() {
        let mut result = 'outer: loop {
            let mut i = 0;
            while i < attrs.len() {
                if attrs[i].path.is_ident("borrows") {
                    break 'outer Some(attrs.remove(i))
                }
            }
            break None
        };
        
        if let Some(result) = result {
            output.push(quote! {
                
            })
        } else {
            output.push(field.to_token_stream());
        }
    }

    return output
}