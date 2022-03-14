use syn::{parse::ParseStream, Attribute, Field};

syn::custom_keyword!(ignore);

#[derive(Default)]
pub struct Attrs {
    pub ignore: bool,
}

impl Attrs {
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
    let attrs = Attrs::new(&field.attrs);

    !attrs.ignore
}
