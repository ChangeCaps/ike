use super::attributes::{ignore_field, FieldAttrs};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, DeriveInput, FieldsUnnamed, Index};

use crate::get_ike::get_ike;

pub fn impl_reflect_tuple(input: &DeriveInput, fields: &FieldsUnnamed) -> TokenStream {
    let ident = &input.ident;

    let ike_reflect = get_ike("reflect");

    let from_reflect = from_reflect(fields);
    let field = field(fields);
    let field_mut = field_mut(fields);
    let field_len = fields.unnamed.iter().filter(ignore_field).count();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #ike_reflect::FromReflect for #ident #ty_generics #where_clause {
            fn from_reflect(
                reflect: &dyn #ike_reflect::Reflect
            ) -> ::std::option::Option<Self> {
                if reflect.type_name() == ::std::any::type_name::<Self>() {
                    let reflect = reflect.reflect_ref().get_tuple()?;

                    ::std::option::Option::Some(Self(
                        #(#from_reflect),*
                    ))
                } else {
                    ::std::option::Option::None
                }
            }
        }

        impl #impl_generics #ike_reflect::ReflectTuple for #ident #ty_generics #where_clause {
            fn field(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&dyn #ike_reflect::Reflect> {
                match index {
                    #(#field,)*
                    _ => None,
                }
            }

            fn field_mut(
                &mut self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&mut dyn #ike_reflect::Reflect> {
                match index {
                    #(#field_mut,)*
                    _ => None,
                }
            }

            fn field_len(&self) -> ::std::primitive::usize {
                #field_len
            }
        }
    }
}

pub fn from_reflect(fields: &FieldsUnnamed) -> impl Iterator<Item = TokenStream> + '_ {
    let ike_reflect = get_ike("reflect");

    fields
        .unnamed
        .iter()
        .enumerate()
        .map(move |(index, field)| {
            let attrs = FieldAttrs::new(&field.attrs);

            let ty = &field.ty;

            if attrs.ignore {
                quote!(::std::default::Default::default())
            } else {
                quote_spanned! {ty.span()=>
                    <#ty as #ike_reflect::FromReflect>::from_reflect(
                        reflect.field(#index).unwrap()
                    )?
                }
            }
        })
}

pub fn field(fields: &FieldsUnnamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .unnamed
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(i, field)| {
            let index = Index {
                index: i as u32,
                span: Span::call_site(),
            };
            let ty = &field.ty;

            quote_spanned! {ty.span()=>
                #i => Some(&self.#index)
            }
        })
}

pub fn field_mut(fields: &FieldsUnnamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .unnamed
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(i, field)| {
            let index = Index {
                index: i as u32,
                span: Span::call_site(),
            };
            let ty = &field.ty;

            quote_spanned! {ty.span()=>
                #i => Some(&mut self.#index)
            }
        })
}
