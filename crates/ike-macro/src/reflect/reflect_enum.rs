use super::attributes::{ignore_field, FieldAttrs};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, DataEnum, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

use crate::get_ike::get_ike;

pub fn impl_reflect_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let ident = &input.ident;

    let ike_reflect = get_ike("reflect");

    let var_pat = var_pat(data).collect::<Vec<_>>();
    let var_type = var_type(data).collect::<Vec<_>>();
    let var_name = var_name(data);

    let from_reflect = from_reflect(data);

    let struct_field = struct_field(data).collect::<Vec<_>>();
    let struct_field_at = struct_field_at(data).collect::<Vec<_>>();
    let struct_name_at = struct_name_at(data);
    let struct_field_len = struct_field_len(data);

    let tuple_field = tuple_field(data).collect::<Vec<_>>();
    let tuple_field_len = tuple_field_len(data);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #ike_reflect::FromReflect for #ident #ty_generics #where_clause {
            fn from_reflect(
                reflect: &dyn #ike_reflect::Reflect
            ) -> ::std::option::Option<Self> {
                if reflect.type_name() == ::std::any::type_name::<Self>() {
                    let reflect = reflect.reflect_ref().get_enum()?;

                    match reflect.variant_name() {
                        #(#from_reflect,)*
                        _ => ::std::option::Option::None,
                    }
                } else {
                    ::std::option::Option::None
                }
            }
        }

        #[allow(unused)]
        impl #impl_generics #ike_reflect::ReflectStruct for #ident #ty_generics #where_clause {
            fn field(
                &self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<&dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #struct_field),*
                }
            }

            fn field_mut(
                &mut self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<&mut dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #struct_field),*
                }
            }

            fn field_at(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #struct_field_at),*
                }
            }

            fn field_at_mut(
                &mut self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&mut dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #struct_field_at),*
                }
            }

            fn name_at(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&::std::primitive::str> {
                match self {
                    #(#var_pat => #struct_name_at),*
                }
            }

            fn field_len(
                &self,
            ) -> ::std::primitive::usize {
                match self {
                    #(#var_pat => #struct_field_len),*
                }
            }
        }

        #[allow(unused)]
        impl #impl_generics #ike_reflect::ReflectTuple for #ident #ty_generics #where_clause {
            fn field(
                &self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #tuple_field),*
                }
            }

            fn field_mut(
                &mut self,
                index: ::std::primitive::usize,
            ) -> ::std::option::Option<&mut dyn #ike_reflect::Reflect> {
                match self {
                    #(#var_pat => #tuple_field),*
                }
            }

            fn field_len(
                &self,
            ) -> ::std::primitive::usize {
                match self {
                    #(#var_pat => #tuple_field_len),*
                }
            }
        }

        #[allow(unused)]
        impl #impl_generics #ike_reflect::ReflectEnum for #ident #ty_generics #where_clause {
            fn variant_name(&self) -> &::std::primitive::str {
                match self {
                    #(#var_pat => #var_name),*
                }
            }

            fn variant_ref(&self) -> #ike_reflect::VariantRef {
                match self {
                    #(#var_pat => #ike_reflect::VariantRef::#var_type(self)),*
                }
            }

            fn variant_mut(&mut self) -> #ike_reflect::VariantMut {
                match self {
                    #(#var_pat => #ike_reflect::VariantMut::#var_type(self)),*
                }
            }
        }
    }
}

fn ident_named(fields: &FieldsNamed) -> impl Iterator<Item = &Ident> {
    fields
        .named
        .iter()
        .filter(ignore_field)
        .map(|field| field.ident.as_ref().unwrap())
}

fn ident_unnamed(fields: &FieldsUnnamed) -> impl Iterator<Item = Ident> + '_ {
    fields
        .unnamed
        .iter()
        .filter(ignore_field)
        .enumerate()
        .map(|(i, _)| Ident::new(&format!("_{}", i), Span::call_site()))
}

fn var_type(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Unit | Fields::Named(_) => quote!(Struct),
        Fields::Unnamed(_) => quote!(Tuple),
    })
}

fn var_name(data: &DataEnum) -> impl Iterator<Item = String> + '_ {
    data.variants
        .iter()
        .map(|variant| variant.ident.to_string())
}

fn var_pat(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| {
        let ident = &variant.ident;

        match variant.fields {
            Fields::Named(ref fields) => {
                let idents = ident_named(fields);

                quote! {
                    Self::#ident { #(#idents,)* }
                }
            }
            Fields::Unnamed(ref fields) => {
                let idents = ident_unnamed(fields);

                quote! {
                    Self::#ident(#(#idents,)*)
                }
            }
            Fields::Unit => quote!(Self::#ident),
        }
    })
}

fn from_reflect(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    let ike_reflect = get_ike("reflect");

    data.variants.iter().map(move |variant| {
        let ident = &variant.ident;
        let name = ident.to_string();

        match variant.fields {
            Fields::Named(ref fields) => {
                let fields = fields.named.iter().map(|field| {
                    let attrs = FieldAttrs::new(&field.attrs);

                    let ident = field.ident.as_ref().unwrap();
                    let name = ident.to_string();
                    let ty = &field.ty;

                    if attrs.ignore {
                        quote!(#ident: ::std::default::Default::default())
                    } else {
                        quote! {
                            #ident: <#ty as #ike_reflect::FromReflect>::from_reflect(
                                reflect.field(#name).unwrap()
                            )?
                        }
                    }
                });

                quote!(#name => {
                    let reflect = reflect.variant_ref().get_struct()?;

                    ::std::option::Option::Some(Self::#ident { #(#fields),* })
                })
            }
            Fields::Unnamed(ref fields) => {
                let fields = fields.unnamed.iter().enumerate().map(|(index, field)| {
                    let attrs = FieldAttrs::new(&field.attrs);

                    let ty = &field.ty;

                    if attrs.ignore {
                        quote!(#ident: ::std::default::Default::default())
                    } else {
                        quote! {
                            <#ty as #ike_reflect::FromReflect>::from_reflect(
                                reflect.field(#index).unwrap()
                            )?
                        }
                    }
                });

                quote!(#name => {
                    let reflect = reflect.variant_ref().get_tuple()?;

                    ::std::option::Option::Some(Self::#ident(#(#fields),*))
                })
            }
            Fields::Unit => quote!(#name => ::std::option::Option::Some(Self::#ident)),
        }
    })
}

fn struct_field(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Named(ref fields) => {
            let field = fields.named.iter().filter(ignore_field).map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let name = ident.to_string();
                let ty = &field.ty;

                quote_spanned! {ty.span()=>
                    #name => ::std::option::Option::Some(#ident)
                }
            });

            quote! {
                match name {
                    #(#field,)*
                    _ => ::std::option::Option::None,
                }
            }
        }
        _ => quote!(::std::option::Option::None),
    })
}

fn struct_field_at(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Named(ref fields) => {
            let field =
                fields
                    .named
                    .iter()
                    .filter(ignore_field)
                    .enumerate()
                    .map(|(index, field)| {
                        let ident = field.ident.as_ref().unwrap();
                        let ty = &field.ty;

                        quote_spanned! {ty.span()=>
                            #index => ::std::option::Option::Some(#ident)
                        }
                    });

            quote! {
                match index {
                    #(#field,)*
                    _ => ::std::option::Option::None,
                }
            }
        }
        _ => quote!(::std::option::Option::None),
    })
}

fn struct_name_at(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Named(ref fields) => {
            let field =
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
                    });

            quote! {
                match index {
                    #(#field,)*
                    _ => ::std::option::Option::None,
                }
            }
        }
        _ => quote!(::std::option::Option::None),
    })
}

fn struct_field_len(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Named(ref fields) => {
            let len = fields.named.iter().filter(ignore_field).count();

            quote!(#len)
        }
        _ => quote!(0usize),
    })
}

fn tuple_field(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Unnamed(ref fields) => {
            let field =
                fields
                    .unnamed
                    .iter()
                    .filter(ignore_field)
                    .enumerate()
                    .map(|(index, field)| {
                        let ident = Ident::new(&format!("_{}", index), Span::call_site());
                        let ty = &field.ty;

                        quote_spanned! {ty.span()=>
                            #index => ::std::option::Option::Some(#ident)
                        }
                    });

            quote! {
                match index {
                    #(#field,)*
                    _ => ::std::option::Option::None,
                }
            }
        }
        _ => quote!(::std::option::Option::None),
    })
}

fn tuple_field_len(data: &DataEnum) -> impl Iterator<Item = TokenStream> + '_ {
    data.variants.iter().map(|variant| match variant.fields {
        Fields::Unnamed(ref fields) => {
            let len = fields.unnamed.iter().filter(ignore_field).count();

            quote!(#len)
        }
        _ => quote!(0usize),
    })
}
