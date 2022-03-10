use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::get_ike::get_ike;

pub fn derive_label(input: proc_macro::TokenStream, trait_ident: Ident) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ike_util = get_ike("util");

    let raw_label =
        raw_label(&input.data).unwrap_or_else(|| quote!(#ike_util::RawLabel::from_hash(self)));

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #trait_ident for #name #ty_generics #where_clause {
            fn raw_label(&self) -> #ike_util::RawLabel {
                #raw_label
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn raw_label(data: &Data) -> Option<TokenStream> {
    let ike_util = get_ike("util");

    match data {
        Data::Enum(data) => {
            let mut variants = Vec::new();

            for variant in data.variants.iter() {
                if variant.fields != Fields::Unit {
                    return None;
                }

                let ident = &variant.ident;
                let name = ident.to_string();

                variants.push(quote! {
                    Self::#ident => {
                        #ike_util::RawLabel::variant::<Self>(::std::borrow::Cow::Borrowed(#name))
                    }
                });
            }

            Some(quote! {
                match self {
                    #(#variants)*
                }
            })
        }
        _ => None,
    }
}
