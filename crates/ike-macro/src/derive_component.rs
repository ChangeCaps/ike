use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::get_ike::get_ike;

pub fn derive_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ike_ecs = get_ike("ecs");
    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #ike_ecs::Registerable for #name #ty_generics #where_clause {
            #[allow(unused_mut)]
            fn type_registration() -> #ike_ecs::TypeRegistration {
                let mut registration = #ike_ecs::TypeRegistration::new::<Self>();

                registration
            }
        }

        impl #impl_generics #ike_ecs::Component for #name #ty_generics #where_clause {
            type Storage = #ike_ecs::SparseStorage;
        }
    };

    proc_macro::TokenStream::from(expanded)
}
