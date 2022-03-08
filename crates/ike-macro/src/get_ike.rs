use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

pub fn get_ike(sub: &str) -> TokenStream {
    if let Ok(found) = crate_name(&format!("ike-{}", sub)) {
        get_crate(found)
    } else if let Ok(found) = crate_name("ike") {
        let path = get_crate(found);

        let ident = Ident::new(sub, Span::call_site());

        quote!(#path::#ident)
    } else {
        panic!("'ike-{}' or 'ike' must be included in Cargo.toml", sub)
    }
}

fn get_crate(found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());

            quote!(::#ident)
        }
    }
}
