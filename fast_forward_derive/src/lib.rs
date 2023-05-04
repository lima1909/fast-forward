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
    let fields: Vec<_> = fields.iter().map(create_field).collect();

    let name = syn::Ident::new(&format!("{name}List"), name.span());
    quote! {
       /// Container-struct for all indices.
       #[derive(Default)]
       pub struct #name {
            #(#fields)*
       }
    }
}

fn create_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_defs: Vec<_> = field
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("index"))
        .map(|a| match a.parse_args::<attr::FieldAttr>() {
            Ok(field_attr) => field_attr.to_tokenstream(field.ident.clone()),
            Err(err) => Error::new_spanned(a, format!("Error by parsing Attribute ({a:?}): {err}"))
                .to_compile_error(),
        })
        .collect();

    quote!( #(#field_defs)* )
}
