use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::ParseStream, parse_macro_input, parse_quote, punctuated::Punctuated, Attribute,
    DeriveInput, Path, Token,
};

use crate::get_ike::get_ike;

#[derive(Default)]
struct DerivedTraits {
    registrations: Vec<Path>,
}

impl DerivedTraits {
    pub fn new(attrs: &[Attribute]) -> Self {
        let mut this = Self::default();

        let ike_ecs = get_ike("ecs");

        for attr in attrs {
            if attr.path.is_ident("derive") {
                let derives = attr
                    .parse_args_with(|input: ParseStream| {
                        Punctuated::<Path, Token![,]>::parse_terminated(input)
                    })
                    .unwrap();

                for derive in derives.iter() {
                    if derive.is_ident("Reflect") {
                        this.registrations
                            .push(parse_quote!(#ike_ecs::ReflectComponent));
                    }
                }
            }
        }

        this
    }

    pub fn registration(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let ike_type = get_ike("type");

        self.registrations.iter().map(move |path| {
            quote! {
                registration.insert(<#path as #ike_type::FromType<Self>>::from_type());
            }
        })
    }
}

pub fn component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ike_ecs = get_ike("ecs");
    let ident = &input.ident;

    let derives = DerivedTraits::new(&input.attrs);
    let registration = derives.registration();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #input

        impl #impl_generics #ike_ecs::Registerable for #ident #ty_generics #where_clause {
            #[allow(unused_mut)]
            fn type_registration() -> #ike_ecs::TypeRegistration {
                let mut registration = #ike_ecs::TypeRegistration::new::<Self>();

                #(#registration)*

                registration
            }
        }

        impl #impl_generics #ike_ecs::Component for #ident #ty_generics #where_clause {
            type Storage = #ike_ecs::SparseStorage;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
