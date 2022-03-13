use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

use crate::get_ike::get_ike;

pub fn derive_system_param(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let fetch_ident = Ident::new(&format!("{}Fetch", ident), ident.span());
    let vis = input.vis;

    let ike_ecs = get_ike("ecs");

    let fetch_fields = fetch_fields(&input.data);
    let fetch_init = fetch_init(&input.data);
    let fetch_access = fetch_access(&input.data);
    let fetch_get = fetch_get(&input.data);
    let register_types = register_types(&input.data);

    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut item_generics = input.generics.clone();
    for lifetime in item_generics.lifetimes_mut() {
        if *lifetime == parse_quote!('w) {
            *lifetime = parse_quote!('__w);
        }

        if *lifetime == parse_quote!('s) {
            *lifetime = parse_quote!('__s);
        }
    }
    let (_, item_generics, _) = item_generics.split_for_impl();

    input.generics.params.insert(0, parse_quote!('__w));
    input.generics.params.insert(1, parse_quote!('__s));
    let (param_generics, _, _) = input.generics.split_for_impl();

    let expanded = quote! {
        #vis struct #fetch_ident #impl_generics {
            #fetch_fields
        }

        impl #param_generics #ike_ecs::SystemParamFetch<'__w, '__s> for #fetch_ident #ty_generics #where_clause {
            type Item = #ident #item_generics;

            fn init(world: &mut #ike_ecs::World) -> Self {
                Self {
                    #fetch_init
                }
            }

            fn access(access: &mut #ike_ecs::SystemAccess) {
                #fetch_access
            }

            fn get(
                &'__s mut self,
                world: &'__w #ike_ecs::World,
                last_change_tick: #ike_ecs::ChangeTick,
            ) -> Self::Item {
                #ident {
                    #fetch_get
                }
            }

            fn apply(self, world: &mut #ike_ecs::World) {}

            fn register_types(type_registry: &mut #ike_ecs::TypeRegistry) {
                #register_types
            }
        }

        impl #impl_generics #ike_ecs::SystemParam for #ident #ty_generics #where_clause {
            type Fetch = #fetch_ident #ty_generics;
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn fetch_fields(data: &Data) -> TokenStream {
    let ike_ecs = get_ike("ecs");

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        pub #ident: <#ty as #ike_ecs::SystemParam>::Fetch,
                    }
                });

                quote! {
                    #(#fields)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn fetch_init(data: &Data) -> TokenStream {
    let ike_ecs = get_ike("ecs");

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        #ident: <#ty as #ike_ecs::SystemParam>::Fetch::init(world),
                    }
                });

                quote! {
                    #(#fields)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn fetch_access(data: &Data) -> TokenStream {
    let ike_ecs = get_ike("ecs");

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ty = &field.ty;

                    quote! {
                        <#ty as #ike_ecs::SystemParam>::Fetch::access(access);
                    }
                });

                quote! {
                    #(#fields)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn fetch_get(data: &Data) -> TokenStream {
    let ike_ecs = get_ike("ecs");

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ty = &field.ty;
                    let ident = &field.ident;

                    quote! {
                        #ident: <#ty as #ike_ecs::SystemParam>::Fetch::get(&mut self.#ident, world, last_change_tick),
                    }
                });

                quote! {
                    #(#fields)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

fn register_types(data: &Data) -> TokenStream {
    let ike_ecs = get_ike("ecs");

    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(ref named) => {
                let fields = named.named.iter().map(|field| {
                    let ty = &field.ty;

                    quote! {
                        <#ty as #ike_ecs::SystemParam>::Fetch::register_types(type_registry);
                    }
                });

                quote! {
                    #(#fields)*
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
