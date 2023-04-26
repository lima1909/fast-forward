use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Indexed, attributes(index))]
pub fn indexed(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // println!("AST:\n {ast:#?}");

    let index = ast.attrs.first().unwrap();
    if let syn::Meta::List(ref l) = index.meta {
        let clone = l.tokens.clone();
        // println!("ATTR:\n {:#?}", l.tokens);
        return quote::quote!( let _c: #clone; ).into();
    }

    quote::quote!(
        impl First {
            fn foo(&self)-> &str {
                &self.name
            }
        }
    )
    .into()
}
