use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Paren,
    Attribute, Data, DataEnum, DeriveInput, Field, Fields, FieldsUnnamed, GenericParam, Generics,
    Index, Path, Token, TypeParamBound,
};

use crate::get_crate;

fn get_reflect() -> TokenStream {
    if let Some(reflect) = get_crate("ike-reflect") {
        quote!(#reflect)
    } else {
        quote!(ike::reflect)
    }
}

syn::custom_keyword!(register);

struct Register {
    _register: register,
    types: Punctuated<Path, Token![,]>,
    _paren: Paren,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            _register: register::parse(input)?,
            _paren: parenthesized!(content in input),
            types: Punctuated::parse_terminated(&content)?,
        })
    }
}

const REFLECT: &str = "reflect";

#[derive(Default)]
struct ContainerArgs {
    value: bool,
    default: bool,
    register: Vec<Path>,
}

impl ContainerArgs {
    #[inline]
    pub fn from_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr
                .path
                .get_ident()
                .map(|ident| ident == REFLECT)
                .unwrap_or(false)
            {
                syn::custom_keyword!(value);
                syn::custom_keyword!(default);

                attr.parse_args_with(|input: ParseStream| {
                    if input.parse::<Option<value>>()?.is_some() {
                        self.value = true;
                    }

                    if input.parse::<Option<default>>()?.is_some() {
                        self.default = true;
                    }

                    if let Ok(register) = Register::parse(input) {
                        for ty in register.types.into_iter() {
                            self.register.push(ty);
                        }
                    }

                    Ok(())
                })
                .expect("invalid args format");
            }
        }
    }
}

#[derive(Default)]
struct FieldArgs {
    ignore: bool,
}

impl FieldArgs {
    #[inline]
    pub fn from_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if attr
                .path
                .get_ident()
                .map(|ident| ident == REFLECT)
                .unwrap_or(false)
            {
                syn::custom_keyword!(ignore);

                attr.parse_args_with(|input: ParseStream| {
                    if input.parse::<Option<ignore>>()?.is_some() {
                        self.ignore = true;
                    }

                    Ok(())
                })
                .expect("invalid args format");
            }
        }
    }
}

pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut args = ContainerArgs::default();
    args.from_attrs(&input.attrs);

    let name = &input.ident;

    let mut generics = input.generics;
    add_trait_bound(&mut generics, parse_quote!(Send));
    add_trait_bound(&mut generics, parse_quote!(Sync));
    add_trait_bound(&mut generics, parse_quote!('static));

    let reflect_impl = impl_reflect(name, &generics, &input.data, &args);

    let expanded = quote!(#reflect_impl);

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bound(generics: &mut Generics, bound: TypeParamBound) {
    for param in &mut generics.params {
        if let GenericParam::Type(ty) = param {
            ty.bounds.push(bound.clone());
        }
    }
}

fn impl_reflect(
    name: &Ident,
    generics: &Generics,
    data: &Data,
    args: &ContainerArgs,
) -> Option<TokenStream> {
    if args.value {
        return Some(impl_value(name, generics, args));
    }

    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => {
                let fields: Vec<_> = named.named.iter().cloned().collect();

                Some(impl_struct(name, generics, &fields, args))
            }
            Fields::Unnamed(unnamed) => {
                let fields: Vec<_> = unnamed.unnamed.iter().cloned().collect();

                Some(impl_tuple_struct(name, generics, &fields, args))
            }
            Fields::Unit => Some(impl_struct(name, generics, &[], args)),
        },
        Data::Enum(data) => Some(impl_enum(name, generics, data)),
        _ => unimplemented!(),
    }
}

fn impl_value(name: &Ident, generics: &Generics, args: &ContainerArgs) -> TokenStream {
    let reflect = get_reflect();

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: Clone));

    let register_types = &args.register;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #reflect::RegisterType for #name #ty_generics #where_clause {
            #[inline]
            fn register(type_registry: &mut #reflect::TypeRegistry) {
                let mut registration = #reflect::TypeRegistration::from_type::<Self>();

                registration.insert::<#reflect::ReflectComponent>(#reflect::FromType::<Self>::from_type());
                registration.insert::<#reflect::ReflectDeserialize>(#reflect::FromType::<Self>::from_type());
                #(registration.insert::<#register_types>(#reflect::FromType::<Self>::from_type());)*

                type_registry.insert(registration);
            }
        }

        impl #impl_generics #reflect::Value for #name #ty_generics #where_clause {
            #[inline]
            fn serialize(&self) -> &dyn #reflect::Serialize {
                self
            }
        }

        unsafe impl #impl_generics #reflect::Reflect for #name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> &str {
                std::any::type_name::<Self>()
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn reflect_ref(&self) -> #reflect::ReflectRef {
                #reflect::ReflectRef::Value(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #reflect::ReflectMut {
                #reflect::ReflectMut::Value(self)
            }

            #[inline]
            fn clone_value(&self) -> Box<dyn #reflect::Reflect> {
                Box::new(self.clone())
            }

            #[inline]
            fn partial_eq(&self, other: &dyn #reflect::Reflect) -> bool {
                if let Some(other) = other.downcast_ref::<Self>() {
                    self == other
                } else {
                    false
                }
            }

            #[inline]
            fn from_reflect(reflect: &dyn #reflect::Reflect) -> Option<Self> {
                if reflect.any().is::<Self>() {
                    reflect.clone_value().downcast().ok().map(|value| *value)
                } else {
                    None
                }
            }

            #[inline]
            fn default_value() -> Self {
                Default::default()
            }
        }
    }
}

fn impl_struct(
    name: &Ident,
    generics: &Generics,
    fields: &[Field],
    args: &ContainerArgs,
) -> TokenStream {
    let reflect = get_reflect();

    let mut ignored = false;

    let fields = fields
        .iter()
        .filter_map(|field| {
            let mut args = FieldArgs::default();

            args.from_attrs(&field.attrs);

            if !args.ignore {
                Some(field)
            } else {
                ignored = true;
                None
            }
        })
        .collect::<Vec<_>>();

    let default = if ignored {
        Some(quote!(..Default::default()))
    } else {
        None
    };

    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect::<Vec<_>>();

    let field_idents = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let field_indices = (0..fields.len()).into_iter().collect::<Vec<_>>();

    let field_count = fields.len();

    let register_types = &args.register;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let default_value = if args.default {
        quote! {
            #[inline]
            fn default_value() -> Self {
                Default::default()
            }
        }
    } else {
        quote! {
            #[inline]
            fn default_value() -> Self {
                Self {
                    #(#field_idents: #reflect::Reflect::default_value(),)*
                    #default
                }
            }
        }
    };

    quote! {
        impl #impl_generics #reflect::RegisterType for #name #ty_generics #where_clause {
            #[inline]
            fn register(type_registry: &mut #reflect::TypeRegistry) {
                let mut registration = #reflect::TypeRegistration::from_type::<Self>();

                registration.insert::<#reflect::ReflectComponent>(#reflect::FromType::<Self>::from_type());
                #(registration.insert::<#register_types>(#reflect::FromType::<Self>::from_type());)*

                type_registry.insert(registration);

                #(<#field_types as #reflect::RegisterType>::register(type_registry);)*
            }
        }

        impl #impl_generics #reflect::Struct for #name #ty_generics #where_clause {
            #[inline]
            fn field(&self, name: &str) -> Option<&dyn #reflect::Reflect> {
                match name {
                    #(#field_names => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn #reflect::Reflect> {
                match name {
                    #(#field_names => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_at(&self, index: usize) -> Option<&dyn #reflect::Reflect> {
                match index {
                    #(#field_indices => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn #reflect::Reflect> {
                match index {
                    #(#field_indices => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn name_at(&self, index: usize) -> Option<&str> {
                match index {
                    #(#field_indices => Some(#field_names),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_len(&self) -> usize {
                #field_count
            }

            #[inline]
            fn clone_dynamic(&self) -> #reflect::DynamicStruct {
                let mut value = #reflect::DynamicStruct::default();

                value.set_name(String::from(std::any::type_name::<Self>()));

                #(
                    value.insert_boxed(#field_names, #reflect::Reflect::clone_value(&self.#field_idents));
                )*

                value
            }
        }

        unsafe impl #impl_generics #reflect::Reflect for #name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> &str {
                std::any::type_name::<Self>()
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn reflect_ref(&self) -> #reflect::ReflectRef {
                #reflect::ReflectRef::Struct(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #reflect::ReflectMut {
                #reflect::ReflectMut::Struct(self)
            }

            #[inline]
            fn clone_value(&self) -> Box<dyn #reflect::Reflect> {
                Box::new(#reflect::Struct::clone_dynamic(self))
            }

            #[inline]
            fn partial_eq(&self, other: &dyn #reflect::Reflect) -> bool {
                match other.reflect_ref() {
                    #reflect::ReflectRef::Struct(other) => {
                        let len = #reflect::Struct::field_len(self);

                        if len == other.field_len() {
                            for i in 0..len {
                                if #reflect::Struct::field_at(self, i)
                                    .unwrap()
                                    .partial_eq(other.field_at(i).unwrap())
                                {
                                    return false;
                                }
                            }

                            true
                        } else {
                            false
                        }
                    }
                    _ => false
                }
            }

            #[inline]
            fn from_reflect(reflect: &dyn #reflect::Reflect) -> Option<Self> {
                if let #reflect::ReflectRef::Struct(value) = reflect.reflect_ref() {
                    Some(Self {
                        #(
                            #field_idents: #reflect::Reflect::from_reflect(value.field(#field_names)?)?,
                        )*
                        #default
                    })
                } else {
                    None
                }
            }

            #default_value
        }
    }
}

fn impl_tuple_struct(
    name: &Ident,
    generics: &Generics,
    fields: &[Field],
    args: &ContainerArgs,
) -> TokenStream {
    let reflect = get_reflect();

    let fields = fields
        .iter()
        .filter_map(|field| {
            let mut args = FieldArgs::default();

            args.from_attrs(&field.attrs);

            if !args.ignore {
                Some(field)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let field_indices = (0..fields.len()).into_iter().collect::<Vec<_>>();

    let field_idents = field_indices
        .iter()
        .map(|index| Index {
            index: *index as u32,
            span: Span::call_site(),
        })
        .collect::<Vec<_>>();

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let field_count = fields.len();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let default_value = if args.default {
        quote! {
            #[inline]
            fn default_value() -> Self {
                Default::default()
            }
        }
    } else {
        quote! {
            #[inline]
            fn default_value() -> Self {
                Self(#(<#field_types as #reflect::Reflect>::default_value(),)*)
            }
        }
    };

    quote! {
        impl #impl_generics #reflect::TupleStruct for #name #ty_generics #where_clause {
            #[inline]
            fn field(&self, index: usize) -> Option<&dyn #reflect::Reflect> {
                match index {
                    #(#field_indices => Some(&self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_mut(&mut self, index: usize) -> Option<&mut dyn #reflect::Reflect> {
                match index {
                    #(#field_indices => Some(&mut self.#field_idents),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_len(&self) -> usize {
                #field_count
            }

            #[inline]
            fn clone_dynamic(&self) -> #reflect::DynamicTupleStruct {
                let mut value = #reflect::DynamicTupleStruct::default();

                value.set_name(String::from(std::any::type_name::<Self>()));

                #(
                    value.push_boxed(#reflect::Reflect::clone_value(&self.#field_idents));
                )*

                value
            }
        }

        unsafe impl #impl_generics #reflect::Reflect for #name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> &str {
                std::any::type_name::<Self>()
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn reflect_ref(&self) -> #reflect::ReflectRef {
                #reflect::ReflectRef::TupleStruct(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #reflect::ReflectMut {
                #reflect::ReflectMut::TupleStruct(self)
            }

            #[inline]
            fn clone_value(&self) -> Box<dyn #reflect::Reflect> {
                Box::new(#reflect::TupleStruct::clone_dynamic(self))
            }

            #[inline]
            fn partial_eq(&self, other: &dyn #reflect::Reflect) -> bool {
                match other.reflect_ref() {
                    #reflect::ReflectRef::TupleStruct(other) => {
                        let len = #reflect::TupleStruct::field_len(self);

                        if len == other.field_len() {
                            for i in 0..len {
                                if #reflect::TupleStruct::field(self, i)
                                    .unwrap()
                                    .partial_eq(other.field(i).unwrap())
                                {
                                    return false;
                                }
                            }

                            true
                        } else {
                            false
                        }
                    }
                    _ => false
                }
            }

            #[inline]
            fn from_reflect(reflect: &dyn #reflect::Reflect) -> Option<Self> {
                if let #reflect::ReflectRef::TupleStruct(value) = reflect.reflect_ref() {
                    Some(Self(
                        #(
                            #reflect::Reflect::from_reflect(value.field(#field_indices)?)?
                        )*
                    ))
                } else {
                    None
                }
            }

            #default_value
        }
    }
}

fn impl_enum(name: &Ident, generics: &Generics, _data: &DataEnum) -> TokenStream {
    let reflect = get_reflect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #reflect::Struct for #name #ty_generics #where_clause {
            #[inline]
            fn field(&self, name: &str) -> Option<&dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_at(&self, index: usize) -> Option<&dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_len(&self) -> usize {

            }
        }

        impl #impl_generics #reflect::TupleStruct for #name #ty_generics #where_clause {
            #[inline]
            fn field(&self, index: usize) -> Option<&dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_mut(&mut self, index: usize) -> Option<&mut dyn #reflect::Reflect> {

            }

            #[inline]
            fn field_len(&self) -> usize {

            }
        }

        unsafe impl #impl_generics #reflect::Reflect for #name #ty_generics #where_clause {
            #[inline]
            fn type_name(&self) -> &str {
                std::any::type_name::<Self>()
            }

            #[inline]
            fn any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[inline]
            fn reflect_ref(&self) -> #reflect::ReflectRef {
                #reflect::ReflectRef::Enum(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #reflect::ReflectMut {
                #reflect::ReflectMut::Enum(self)
            }

            #[inline]
            fn clone_value(&self) -> Box<dyn #reflect::Reflect> {

            }
        }
    }
}
