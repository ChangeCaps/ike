use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

pub fn ike_main(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ike = quote!(ike);
    let input = parse_macro_input!(input as ItemFn);

    if input.sig.ident != "main" {
        panic!("main function must be named 'main'");
    }

    let expanded = quote_spanned! {input.sig.inputs.span()=>
        #[cfg(not(editor))]
        fn main() {
            #input

            let mut app = #ike::core::App::new();

            main(&mut app);

            app.run();
        }

        #[cfg(editor)]
        unsafe extern "Rust" fn ike_main(app: &mut #ike::core::AppBuilder) {
            #input

            main(&mut app);
        }
    };

    proc_macro::TokenStream::from(expanded)
}
