use attr::FieldAttrs;
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
    let mut index_fields = Vec::new();

    match fields {
        syn::Fields::Named(f) => {
            let _it = f.named.iter();
        }
        syn::Fields::Unnamed(f) => {
            let _it = f.unnamed.iter();
        }
        syn::Fields::Unit => {
            // TODO error
        }
    }

    for field in fields {
        match field_to_attrs(field) {
            Ok(attrs) => {
                if let Some(attrs) = attrs {
                    index_fields.push(attrs.to_tokenstream())
                }
            }
            Err(err) => return err.to_compile_error(),
        }
    }

    let name = syn::Ident::new(&format!("{name}List"), name.span());
    quote! {
       /// Container-struct for all indices.
       #[derive(Default)]
       pub struct #name {
            #(#index_fields)*
       }
    }
}

fn field_to_attrs(field: &syn::Field) -> syn::Result<Option<FieldAttrs>> {
    if field.attrs.is_empty() {
        return Ok(None);
    }

    let mut attrs = FieldAttrs::new(field.clone());
    for attr in field.attrs.iter().filter(|a| a.path().is_ident("index")) {
        match attr.parse_args::<attr::Attr>() {
            Ok(attr) => attrs.add(attr),
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(Some(attrs))
}
