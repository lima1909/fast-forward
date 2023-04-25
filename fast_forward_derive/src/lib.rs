use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Indexed)]
pub fn indexed(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    println!("AST:\n {ast:#?}");

    TokenStream::new()
}
