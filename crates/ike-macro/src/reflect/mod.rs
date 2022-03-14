mod attributes;
mod reflect_enum;
mod reflect_struct;
mod reflect_tuple;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, Generics, TypeParamBound};

use crate::get_ike::get_ike;

use self::{
    reflect_enum::impl_reflect_enum,
    reflect_struct::{impl_reflect_struct, impl_reflect_unit_struct},
    reflect_tuple::impl_reflect_tuple,
};

pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let ike_reflect = get_ike("reflect");
    add_trait_bound(&mut input.generics, parse_quote!(#ike_reflect::Reflect));

    let reflect_impl = reflect_impl(&input);

    let ty = reflect_type(&input.data);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #reflect_impl

        impl #impl_generics #ike_reflect::Reflect for #ident #ty_generics #where_clause {
            fn reflect_ref(&self) -> #ike_reflect::ReflectRef {
                #ike_reflect::ReflectRef::#ty(self)
            }

            fn reflect_mut(&mut self) -> #ike_reflect::ReflectMut {
                #ike_reflect::ReflectMut::#ty(self)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bound(generics: &mut Generics, bound: TypeParamBound) {
    for param in generics.type_params_mut() {
        param.bounds.push(bound.clone());
    }
}

fn reflect_type(data: &Data) -> Ident {
    match data {
        Data::Enum(_) => parse_quote!(Enum),
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) | Fields::Unit => parse_quote!(Struct),
            Fields::Unnamed(_) => parse_quote!(Tuple),
        },
        _ => unimplemented!("Reflect cannot be derived for unions"),
    }
}

fn reflect_impl(input: &DeriveInput) -> TokenStream {
    match input.data {
        Data::Enum(ref data) => impl_reflect_enum(input, data),
        Data::Struct(ref data) => match data.fields {
            Fields::Unnamed(ref fields) => impl_reflect_tuple(input, fields),
            Fields::Named(ref fields) => impl_reflect_struct(input, fields),
            Fields::Unit => impl_reflect_unit_struct(input),
        },
        _ => unimplemented!("Reflect cannot be derived for unions"),
    }
}
