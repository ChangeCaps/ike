use get_ike::get_ike;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod derive_component;
mod get_ike;

#[proc_macro_derive(Component)]
pub fn derive_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_component::derive_component(input)
}

macro_rules! derive_label {
    ($label:ident, $fn:ident) => {
        #[proc_macro_derive($label)]
        pub fn $fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            let input = parse_macro_input!(input as DeriveInput);

            let ike_ecs = get_ike("ecs");
            let ike_id = get_ike("id");
            let name = input.ident;

            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

            let expanded = quote! {
                impl #impl_generics #ike_ecs::$label for #name #ty_generics #where_clause {
                    fn raw_label(&self) -> #ike_id::RawLabel {
                        #ike_id::RawLabel::from_hash(self)
                    }
                }
            };

            proc_macro::TokenStream::from(expanded)
        }
    };
}

derive_label!(SystemLabel, derive_system_label);
derive_label!(StageLabel, derive_stage_label);
