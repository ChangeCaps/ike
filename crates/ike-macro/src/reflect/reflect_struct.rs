use super::attributes::{ignore_field, FieldAttrs};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, DeriveInput, FieldsNamed};

use crate::get_ike::get_ike;

pub fn impl_reflect_struct(input: &DeriveInput, fields: &FieldsNamed) -> TokenStream {
    let ident = &input.ident;

    let ike_reflect = get_ike("reflect");

    let from_reflect = from_reflect(fields);
    let field = field(fields);
    let field_mut = field_mut(fields);
    let field_at = field_at(fields);
    let field_at_mut = field_at_mut(fields);
    let name_at = name_at(fields);
    let field_len = field_len(fields);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #ike_reflect::FromReflect for #ident #ty_generics #where_clause {
            fn from_reflect(
                reflect: &dyn #ike_reflect::Reflect
            ) -> ::std::option::Option<Self> {
                if reflect.type_name() == ::std::any::type_name::<Self>() {
                    let reflect = reflect.reflect_ref().get_struct()?;

                    ::std::option::Option::Some(Self {
                        #(#from_reflect),*
                    })
                } else {
                    ::std::option::Option::None
                }
            }
        }

        impl #impl_generics #ike_reflect::ReflectStruct for #ident #ty_generics #where_clause {
            fn field(
                &self,
                name: &::std::primitive::str,
            ) -> Option<&dyn #ike_reflect::Reflect> {
                match name {
                    #(#field,)*
                    _ => None,
                }
            }

            fn field_mut(
                &mut self,
                name: &::std::primitive::str,
            ) -> Option<&mut dyn #ike_reflect::Reflect> {
                match name {
                    #(#field_mut,)*
                    _ => None,
                }
            }

            fn field_at(
                &self,
                index: ::std::primitive::usize,
            ) -> Option<&dyn #ike_reflect::Reflect> {
                match index {
                    #(#field_at,)*
                    _ => None,
                }
            }

            fn field_at_mut(
                &mut self,
                index: ::std::primitive::usize,
            ) -> Option<&mut dyn #ike_reflect::Reflect> {
                match index {
                    #(#field_at_mut,)*
                    _ => None,
                }
            }

            fn name_at(
                &self,
                index: ::std::primitive::usize,
            ) -> Option<&::std::primitive::str> {
                match index {
                    #(#name_at,)*
                    _ => None,
                }
            }

            fn field_len(&self) -> ::std::primitive::usize {
                #field_len
            }
        }
    }
}

pub fn from_reflect(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    let ike_reflect = get_ike("reflect");

    fields.named.iter().map(move |field| {
        let attrs = FieldAttrs::new(&field.attrs);

        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        let ty = &field.ty;

        if attrs.ignore {
            quote!(#ident: ::std::default::Default::default())
        } else {
            quote_spanned! {ty.span()=>
                #ident: <#ty as #ike_reflect::FromReflect>::from_reflect(
                    reflect.field(#name).unwrap()
                )?
            }
        }
    })
}

pub fn field(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.named.iter().filter(ignore_field).map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        let ty = &field.ty;

        quote_spanned! {ty.span()=>
            #name => ::std::option::Option::Some(&self.#ident)
        }
    })
}

pub fn field_mut(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.named.iter().filter(ignore_field).map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        let ty = &field.ty;

        quote_spanned! {ty.span()=>
            #name => ::std::option::Option::Some(&mut self.#ident)
        }
    })
}

pub fn field_at(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .named
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(index, field)| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            quote_spanned! {ty.span()=>
                #index => ::std::option::Option::Some(&self.#ident)
            }
        })
}

pub fn field_at_mut(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .named
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(index, field)| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            quote_spanned! {ty.span()=>
                #index => ::std::option::Option::Some(&mut self.#ident)
            }
        })
}

fn name_at(fields: &FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields
        .named
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(index, field)| {
            let ident = field.ident.as_ref().unwrap();
            let name = ident.to_string();
            let ty = &field.ty;

            quote_spanned! {ty.span()=>
                #index => ::std::option::Option::Some(#name)
            }
        })
}

fn field_len(fields: &FieldsNamed) -> TokenStream {
    let len = fields.named.iter().filter(ignore_field).count();

    quote!(#len)
}

pub fn impl_reflect_unit_struct(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    let ike_reflect = get_ike("reflect");

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #ike_reflect::FromReflect for #ident #ty_generics #where_clause {
            fn from_reflect(reflect: &dyn #ike_reflect::Reflect) -> Option<Self> {
                if reflect.type_name() == ::std::any::type_name::<Self>() {
                    Some(Self)
                } else {
                    None
                }
            }
        }

        impl #impl_generics #ike_reflect::ReflectStruct for #ident #ty_generics #where_clause {
            fn field(
                &self,
                name: &::std::primitive::str,
            ) -> Option<&dyn #ike_reflect::Reflect> {
                None
            }

            fn field_mut(
                &mut self,
                name: &::std::primitive::str,
            ) -> Option<&mut dyn #ike_reflect::Reflect> {
                None
            }

            fn field_at(
                &self,
                index: ::std::primitive::usize,
            ) -> Option<&dyn #ike_reflect::Reflect> {
                None
            }

            fn field_at_mut(
                &mut self,
                index: ::std::primitive::usize,
            ) -> Option<&mut dyn #ike_reflect::Reflect> {
                None
            }

            fn name_at(
                &self,
                index: ::std::primitive::usize,
            ) -> Option<&::std::primitive::str> {
                None
            }

            fn field_len(&self) -> ::std::primitive::usize {
                0usize
            }
        }
    }
}
