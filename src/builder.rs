use std::{cell::{RefCell, Ref}, ops::Deref};
use derive_syn_parse::Parse;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens, format_ident};
use syn::{FieldsNamed, Fields, ItemStruct, FieldsUnnamed, punctuated::Punctuated, Token, Field, parse_quote, Type, TypeReference};
use crate::{to_snake_case, attrs::SisAttr};

#[inline]
pub fn self_referencing_impl (sis_attrs: &Punctuated<SisAttr, Token![,]>, ItemStruct { attrs, vis, struct_token, ident, mut generics, fields, .. }: ItemStruct) -> TokenStream {
    generics.params.push(parse_quote! { 'this });
    let fields = match fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        _ => todo!()
    }.into_iter().map(RefCell::new).collect::<Punctuated<_, Token![,]>>();

    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();
    let [new, field_def, field_new, getter, init_arg, init, try_init_arg, try_init, drop] = match builder_fields(&fields) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error()
    };

    let field_names = fields.iter()
        .map(|x| Ref::map(x.borrow(), |x| &x.ident))
        .collect::<Vec<_>>();

    let field_names = field_names.iter().map(Deref::deref).collect::<Vec<_>>();
    let macro_name = format_ident!("new_{}", to_snake_case(&ident.to_string()));
    let macro_try_name = format_ident!("try_{}", macro_name);
    let const_token = sis_attrs.iter().find_map(SisAttr::as_const);

    return quote! {
        #(#attrs)*
        #vis #struct_token #ident #impl_generics #where_generics {
            #(
                #field_def
            ),*
        }

        impl #impl_generics #ident #ty_generics #where_generics {
            #[doc(hidden)]
            #[inline]
            #vis #const_token unsafe fn _new_uninit (#(#new),*) -> Self {
                return Self {
                    #(
                        #field_new
                    ),*
                }
            }

            #[doc(hidden)]
            #vis unsafe fn _initialize (self: ::core::pin::Pin<&'this mut Self>, #(#init_arg),*) {
                let Self { #(#field_names,)* _pin }: &'this mut Self = ::core::pin::Pin::into_inner_unchecked(self);
                #(#init)*
            }

            #[doc(hidden)]
            #vis unsafe fn _try_initialize<E> (self: ::core::pin::Pin<&'this mut Self>, #(#try_init_arg),*) -> ::core::result::Result<(), E> {
                let Self { #(#field_names,)* _pin }: &'this mut Self = ::core::pin::Pin::into_inner_unchecked(self);
                #(#try_init)*
                return Ok(())
            }

            #(#getter)*
        }

        impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_generics {
            fn drop (&mut self) {
                unsafe {
                    #(#drop)*
                }
            }
        }

        macro_rules! #macro_name {
            ({ $($new:expr),* }, { $($init:expr),* }, $name:ident) => {
                let mut $name = unsafe { <#ident>::_new_uninit($($new),*) };
                unsafe {
                    let $name = &mut *::core::ptr::addr_of_mut!($name);
                    let $name = core::pin::Pin::new_unchecked($name);
                    $name._initialize($($init),*);
                }
                // Shadow the original binding so that it can't be directly accessed
                // ever again.
                #[allow(unused_mut)]
                let mut $name = unsafe { core::pin::Pin::new_unchecked(&mut $name) };
            };

            ({ $($new:expr),* }, { $($init:expr),* }, box $name:ident) => {
                let mut $name = unsafe { std::boxed::Box::new(<#ident>::_new_uninit($($new),*)) };
                unsafe {
                    let $name = &mut *(::core::ops::DerefMut::deref_mut(&mut $name) as *mut _);
                    <#ident>::_initialize(core::pin::Pin::new_unchecked($name), $($init),*);
                }
                #[allow(unused_mut)]
                let mut $name = std::boxed::Box::into_pin($name);
            };

            ({ $($new:expr),* }, { $($init:expr),* }, box $name:ident in $alloc:expr) => {
                let mut $name = unsafe { std::boxed::Box::new_in(<#ident>::_new_uninit($($new),*), $alloc) };
                unsafe {
                    let $name = &mut *(::core::ops::DerefMut::deref_mut(&mut $name) as *mut _);
                    <#ident>::_initialize(core::pin::Pin::new_unchecked($name), $($init),*);
                }
                #[allow(unused_mut)]
                let mut $name = std::boxed::Box::into_pin($name);
            };
        }

        macro_rules! #macro_try_name {
            ({ $($new:expr),* }, { $($init:expr),* }, $name:ident) => {
                let mut $name = unsafe { <#ident>::_new_uninit($($new),*) };
                unsafe {                    
                    if let Err(e) = (core::pin::Pin::new_unchecked(&mut *::core::ptr::addr_of_mut!($name)))._try_initialize($($init),*) {
                        ::core::mem::forget($name);
                        return Err(e)
                    }
                }
                // Shadow the original binding so that it can't be directly accessed
                // ever again.
                #[allow(unused_mut)]
                let mut $name = unsafe { core::pin::Pin::new_unchecked(&mut $name) };
            };

            ({ $($new:expr),* }, { $($init:expr),* }, box $name:ident) => {
                let mut $name = unsafe { std::boxed::Box::new(<#ident>::_new_uninit($($new),*)) };
                unsafe {
                    if let Err(e) = <#ident>::_try_initialize(core::pin::Pin::new_unchecked(&mut *(::core::ops::DerefMut::deref_mut(&mut $name) as *mut _)), $($init),*) {
                        core::mem::forget($name);
                    }
                }
                #[allow(unused_mut)]
                let mut $name = std::boxed::Box::into_pin($name);
            };

            ({ $($new:expr),* }, { $($init:expr),* }, box $name:ident in $alloc:expr) => {
                let mut $name = unsafe { std::boxed::Box::new_in(<#ident>::_new_uninit($($new),*), $alloc) };
                unsafe {
                    if let Err(e) = <#ident>::_try_initialize(core::pin::Pin::new_unchecked(&mut *(::core::ops::DerefMut::deref_mut(&mut $name) as *mut _)), $($init),*) {
                        core::mem::forget($name);
                    }
                }
                #[allow(unused_mut)]
                let mut $name = std::boxed::Box::into_pin($name);
            };
        }
    }
}

fn builder_fields (fields: &Punctuated<RefCell<Field>, Token![,]>) -> syn::Result<[Vec<TokenStream>; 9]> {
    let mut new_output = Vec::with_capacity(fields.len());
    let mut field_def_output = Vec::with_capacity(fields.len());
    let mut field_new_output = Vec::with_capacity(fields.len());
    let mut getter_output = Vec::with_capacity(fields.len());
    let mut init_arg_output = Vec::with_capacity(fields.len());
    let mut try_init_arg_output = Vec::with_capacity(fields.len());
    let mut init_output = Vec::with_capacity(fields.len());
    let mut try_init_output = Vec::with_capacity(fields.len());
    let mut drop_output = Vec::with_capacity(fields.len());

    let mut previous_fields = Vec::with_capacity(fields.len());
    for (i, field) in fields.iter().enumerate() {
        let mut field_mut = field.borrow_mut();
        let attrs = &mut field_mut.attrs;
        let result = 'outer: loop {
            let mut i = 0;
            while i < attrs.len() {
                if attrs[i].path.is_ident("borrows") {
                    break 'outer Some(attrs.remove(i))
                }
                i += 1
            }
            break None
        };

        drop(field_mut);
        let field = field.borrow();
        let field = &field as &Field;
        
        if let Some(result) = result {
            let tokens = proc_macro::TokenStream::from(result.tokens);
            let BorrowInput { targets, .. } = syn::parse_macro_input::parse::<BorrowInput>(tokens)?;
            let Field { attrs, vis, ident, colon_token, ty } = field;

            let (target_mut, target_ident) = targets.into_iter()
                .map(|x| (x.mutability, x.ident))
                .unzip::<_, _, Vec<_>, Vec<_>>();

            field_def_output.push(quote! {
                #(#attrs)* #ident #colon_token ::core::mem::MaybeUninit<#ty>
            });

            field_new_output.push(quote! {
                #ident #colon_token ::core::mem::MaybeUninit::uninit()
            });

            // Shared getter
            let getter_ident = match ident {
                Some(x) => x.clone(),
                None => format_ident!("v{i}")
            };

            let (getter_ty, getter) = match ty {
                Type::Reference(TypeReference { and_token, elem, .. }) => (
                    quote! { #and_token #elem },
                    quote! { self.#getter_ident.assume_init_read() }
                ),
                ty => (
                    quote! { &#ty },
                    quote! { self.#getter_ident.assume_init_ref() }
                )
            };
            getter_output.push(quote! {
                #[inline]
                #vis fn #getter_ident (&self) -> #getter_ty {
                    unsafe { return #getter }
                }
            });

            // Mutable getter
            let getter_mut_ident = format_ident!("{getter_ident}_mut");
            getter_output.push(quote! {
                #[inline]
                #vis fn #getter_mut_ident (&mut self) -> &mut #ty {
                    unsafe { return self.#getter_ident.assume_init_mut() }
                }
            });

            // Create ident for initializing function
            let init_f = format_ident!("f_{getter_ident}");

            // Get target fields types
            let mut target_ty = Vec::with_capacity(target_ident.len());
            for ident in target_ident.iter() {
                for field in fields.iter().map(RefCell::borrow) {
                    if field.ident.as_ref() == Some(ident) {
                        target_ty.push(Ref::map(field, |x| &x.ty));
                        break
                    }
                }
            }
            let target_ty = target_ty.iter()
                .map(Deref::deref)
                .collect::<Vec<_>>();

            // Initialization argument
            let mut init_args = Vec::with_capacity(target_mut.len());
            let mut init_pinning_args = Vec::with_capacity(target_mut.len());
            for (target_mut, target_ty) in target_mut.iter().zip(target_ty.iter()) {
                let (ty, pinning) = match target_mut {
                    Some(target_mut) => (
                        quote! { ::core::pin::Pin<&'this #target_mut #target_ty> },
                        quote! {
                            #(
                                let #target_ident = ::core::pin::Pin::new_unchecked(#target_ident as &'this #target_mut #target_ty)
                            )*
                        }
                    ),
                    
                    None => (
                        quote! { &'this #target_ty },
                        quote! { 
                            #(
                                let #target_ident = #target_ident as &'this #target_ty;
                            )*
                        }
                    )
                };
                
                init_args.push(ty);
                init_pinning_args.push(pinning);
            }

            // Regular initializer
            init_arg_output.push(quote! {
                #init_f: impl ::core::ops::FnOnce(
                    #(#init_args),*
                ) -> #ty
            });
            init_output.push(quote! {{
                #(#init_pinning_args)*
                #ident.write(#init_f (#(#target_ident),*));
            }});

            // Fallible initializer
            try_init_arg_output.push(quote! {
                #init_f: impl ::core::ops::FnOnce(
                    #(#init_args),*
                ) -> ::core::result::Result<#ty, E>
            });
            try_init_output.push(quote! {{
                #(#init_pinning_args)*
                match (#init_f (#(#target_ident),*)) {
                    Ok(x) => #ident.write(x),
                    Err(e) => {
                        #(
                            #previous_fields.assume_init_drop();
                        )*
                        return Err(e)
                    }
                };
            }});

            // Destructor
            drop_output.push(quote! {
                if ::core::mem::needs_drop::<#ty>() {
                    self.#getter_ident.assume_init_drop()
                }
            });

            previous_fields.push(getter_ident);
        } else {
            let Field { ident, colon_token, ty, .. } = &field;
            new_output.push(quote! { #ident #colon_token #ty });
            field_def_output.push(field.to_token_stream());
            field_new_output.push(ident.to_token_stream());
        }
    }

    field_def_output.push(quote! {
        _pin: ::core::marker::PhantomPinned
    });

    field_new_output.push(quote! {
        _pin: ::core::marker::PhantomPinned
    });

    return Ok([
        new_output,
        field_def_output,
        field_new_output,
        getter_output,
        init_arg_output,
        init_output,
        try_init_arg_output,
        try_init_output,
        drop_output
    ])
}

#[derive(Parse)]
struct BorrowInput {
    #[paren]
    _paren_token: syn::token::Paren,
    #[inside(_paren_token)]
    #[call(Punctuated::parse_terminated)]
    targets: Punctuated<Borrow, Token![,]>
}

#[derive(Parse)]
struct Borrow {
    mutability: Option<Token![mut]>,
    ident: Ident
}