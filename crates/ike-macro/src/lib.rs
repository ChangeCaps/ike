use syn::parse_quote;

mod derive_component;
mod get_ike;
mod label;
mod node;
mod reflect;
mod system_param;

#[proc_macro_derive(Component)]
pub fn derive_component(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_component::derive_component(input)
}

#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    reflect::derive_reflect(input)
}

#[proc_macro_attribute]
pub fn node(
    _attributes: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    node::node(input)
}

#[proc_macro_derive(SystemParam)]
pub fn derive_system_param(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    system_param::derive_system_param(input)
}

macro_rules! derive_label {
    ($label:ident, $fn:ident) => {
        #[proc_macro_derive($label)]
        pub fn $fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            label::derive_label(input, parse_quote!($label))
        }
    };
}

derive_label!(SystemLabel, derive_system_label);
derive_label!(StageLabel, derive_stage_label);
