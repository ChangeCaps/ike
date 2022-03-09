use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, ImplItem, ImplItemMethod, ItemImpl,
    TraitItemMethod,
};

use crate::get_ike::get_ike;

pub fn node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    if input.trait_.is_some() {
        panic!("expected no trait in impl");
    }

    let name = &input.self_ty;

    let ike_node = get_ike("node");
    let ike_ecs = get_ike("ecs");

    let mut node_generics = input.generics.clone();
    node_generics
        .params
        .insert(0, parse_quote!('__node_lifetime));

    let (node_impl_generics, _node_ty_generics, node_where_clause) = node_generics.split_for_impl();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let node_functions = node_functions(&input.items).collect::<Vec<_>>();

    let node_trait_functions = node_trait_functions(&node_functions);
    let node_function_impls = node_function_impls(&node_functions);
    let add_node_functions = add_node_functions(&node_functions);

    let expanded = quote! {
        #[allow(unused)]
        #input

        const _: () = {
            pub trait __NodeTrait {
                #(#node_trait_functions)*
            }


            impl #node_impl_generics __NodeTrait for
                #ike_ecs::CompMut<'__node_lifetime, self::#name #ty_generics>
                #node_where_clause
            {
                #(#node_function_impls)*
            }

            impl #impl_generics #ike_node::NodeComponent for #name #ty_generics #where_clause {
                fn stages() -> &'static [#ike_node::NodeFn<Self>] {
                    &[#(#add_node_functions),*]
                }
            }
        };
    };

    proc_macro::TokenStream::from(expanded)
}

struct NodeFunction {
    name: String,
    node_ident: Ident,
    node_trait_function: TraitItemMethod,
    node_function: ImplItemMethod,
    span: Span,
}

impl NodeFunction {
    pub fn new(item: &ImplItemMethod) -> Self {
        let name = item.sig.ident.to_string();
        let node_name = format!("__node_{}", name);
        let node_ident = Ident::new(&node_name, item.sig.ident.span());
        let mut sig = item.sig.clone();

        if sig.inputs.iter().next() != Some(&parse_quote!(&mut self)) {
            panic!("node functions must be &mut self");
        }

        sig.ident = node_ident.clone();
        sig.inputs[0] = parse_quote!(self);

        let node_trait_function = parse_quote! {
            #sig;
        };

        sig.inputs[0] = parse_quote!(mut self);
        let block = &item.block;

        let node_function = parse_quote! {
            #sig #block
        };

        Self {
            name,
            node_ident,
            node_trait_function,
            node_function,
            span: item.span(),
        }
    }
}

fn node_functions(items: &[ImplItem]) -> impl Iterator<Item = NodeFunction> + '_ {
    items.iter().filter_map(move |item| match item {
        ImplItem::Method(item) => Some(NodeFunction::new(item)),
        _ => None,
    })
}

fn add_node_functions(functions: &[NodeFunction]) -> impl Iterator<Item = TokenStream> + '_ {
    let ike_ecs = get_ike("ecs");
    let ike_node = get_ike("node");

    functions.iter().map(move |function| {
        let name = &function.name;
        let node_ident = &function.node_ident;

        quote_spanned! {function.span=>
            #ike_node::NodeFn {
                name: #name,
                func: |component, node| {
                    <#ike_ecs::CompMut<Self> as __NodeTrait>::#node_ident(component, node);
                },
            }
        }
    })
}

fn node_trait_functions(functions: &[NodeFunction]) -> impl Iterator<Item = TokenStream> + '_ {
    functions
        .iter()
        .map(|function| function.node_trait_function.clone().into_token_stream())
}

fn node_function_impls(functions: &[NodeFunction]) -> impl Iterator<Item = TokenStream> + '_ {
    functions
        .iter()
        .map(|function| function.node_function.clone().into_token_stream())
}
