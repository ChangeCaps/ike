use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr};
use uuid::Uuid;

use crate::get_ike::get_ike;

pub fn type_uuid(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let type_uuid = parse_macro_input!(args as LitStr).value();
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let expanded = if let Ok(uuid) = Uuid::parse_str(&type_uuid) {
        let ike_util = get_ike("util");

        let uuid_u128 = uuid.as_u128();

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        quote! {
            impl #impl_generics #ike_util::TypeUuid for #ident #ty_generics #where_clause {
                const TYPE_UUID: #ike_util::Uuid = #ike_util::Uuid::from_u128(#uuid_u128);
            }

            #input
        }
    } else {
        let invalid = format!("invalid uuid '{}'", type_uuid);

        quote! {
            const _: () = ::std::compile_error!(#invalid);

            #input
        }
    };

    proc_macro::TokenStream::from(expanded)
}
