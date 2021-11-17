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
        #[allow(unused)]
        fn main() {
            #input

            let mut app = #ike::core::App::new();

            main(&mut app);

            app.run();
        }

        #[cfg(editor)]
        #[no_mangle]
        unsafe extern "Rust" fn ike_main(
            render_ctx: std::sync::Arc<#ike::render::RenderCtx>,
            render_surface: #ike::render::RenderSurface,
        ) -> #ike::core::DynamicApp {
            #input

            #ike::render::set_render_ctx(render_ctx);

            let mut app = #ike::core::App::new();

            app.insert_resource(render_surface);

            main(&mut app);

            Box::new(app.build())
        }
    };

    proc_macro::TokenStream::from(expanded)
}
