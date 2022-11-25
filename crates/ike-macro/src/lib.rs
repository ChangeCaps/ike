use proc_macro_crate::FoundCrate;
use syn::parse_quote;

fn find_shiv() -> syn::Path {
    match proc_macro_crate::crate_name("shiv") {
        Ok(FoundCrate::Itself) => unreachable!(),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            return parse_quote!(::#ident);
        }
        Err(_) => {}
    }

    match proc_macro_crate::crate_name("ike-ecs") {
        Ok(FoundCrate::Itself) => unreachable!(),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            return parse_quote!(::#ident);
        }
        Err(_) => {}
    }

    match proc_macro_crate::crate_name("ike") {
        Ok(found) => match found {
            FoundCrate::Itself => parse_quote!(ike::ecs),
            FoundCrate::Name(name) => {
                let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
                parse_quote!(::#ident::ecs)
            }
        },
        Err(_) => {
            parse_quote!(ike::ecs)
        }
    }
}

shiv_macro_impl::implement!(find_shiv());
