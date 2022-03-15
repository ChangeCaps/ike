use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, Field, LitStr, Token, WherePredicate,
};

syn::custom_keyword!(bound);
syn::custom_keyword!(ignore);
syn::custom_keyword!(Component);

#[derive(Default)]
pub struct FieldAttrs {
    pub ignore: bool,
}

impl FieldAttrs {
    pub fn new(attrs: &[Attribute]) -> Self {
        let mut this = Self::default();

        for attr in attrs {
            if attr.path.is_ident("reflect") {
                attr.parse_args_with(|parser: ParseStream| {
                    if parser.parse::<ignore>().is_ok() {
                        this.ignore = true;
                    }

                    Ok(())
                })
                .unwrap();
            }
        }

        this
    }
}

pub fn ignore_field(field: &&Field) -> bool {
    let attrs = FieldAttrs::new(&field.attrs);

    !attrs.ignore
}

#[allow(unused)]
struct BoundAttr {
    _bound: bound,
    equal: Token![=],
    bound: LitStr,
}

impl Parse for BoundAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _bound: input.parse()?,
            equal: input.parse()?,
            bound: input.parse()?,
        })
    }
}

#[derive(Default)]
pub struct ContainerAttrs {
    pub bound: Option<Punctuated<WherePredicate, Token![,]>>,
    pub component: bool,
}

impl ContainerAttrs {
    pub fn new(attrs: &[Attribute]) -> Self {
        let mut this = Self::default();

        for attr in attrs {
            if attr.path.is_ident("reflect") {
                attr.parse_args_with(|parser: ParseStream| {
                    if let Ok(bound_attr) = parser.parse::<BoundAttr>() {
                        let bound = Parser::parse_str(
                            |parse_buffer: ParseStream| Punctuated::parse_terminated(parse_buffer),
                            &bound_attr.bound.value(),
                        )
                        .unwrap();

                        this.bound = Some(bound);
                    }

                    if parser.parse::<Component>().is_ok() {
                        this.component = true;
                    }

                    Ok(())
                })
                .unwrap();
            }
        }

        this
    }
}
