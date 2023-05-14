use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

mod attr;

#[proc_macro_derive(Indexed, attributes(index))]
pub fn indexed(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        syn::Data::Struct(s) => create_struct(&ast.ident, &s.fields).into(),
        syn::Data::Enum(_) => Error::new_spanned(ast, "Enum are not supported for Index Lists")
            .to_compile_error()
            .into(),
        syn::Data::Union(_) => Error::new_spanned(ast, "Union are not supported for Index Lists")
            .to_compile_error()
            .into(),
    }
}

fn create_struct(name: &syn::Ident, fields: &syn::Fields) -> proc_macro2::TokenStream {
    let attrs_fields: Result<Vec<proc_macro2::TokenStream>, Error> = fields
        .iter()
        .map(|field| attr::from_field(field.clone()))
        .collect();

    match attrs_fields {
        Ok(attrs) => {
            let name = syn::Ident::new(&format!("{name}List"), name.span());

            quote! {
               #[derive(Default)]
               pub struct #name {
                    #(#attrs)*
               }
            }
        }
        Err(err) => err.to_compile_error(),
    }
}
