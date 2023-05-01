use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, DeriveInput, Error};

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
    let fs: Vec<_> = fields.iter().map(create_field).collect();

    let name = syn::Ident::new(&format!("{name}List"), name.span());
    quote! {
       /// Container-struct for all indices.
       #[derive(Default)]
       pub struct #name {
            #(#fs)*
       }
    }
}

fn create_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_defs: Vec<_> = field
        .attrs
        .iter()
        .filter(|a| {
            a.path().is_ident("index")
            // if a.path().is_ident("index") {
            //     if let syn::Meta::List(ref l) = a.meta {
            //         return l.path.is_ident("index");
            //         // let s = a.parse_args::<Store>().unwrap();
            //     }
            // }
            // false
        })
        .map(|a| match a.parse_args::<FieldAttr>() {
            Ok(field_attr) => field_attr.to_tokenstream(field.ident.clone()),
            Err(err) => Error::new_spanned(a, err).to_compile_error(),
        })
        .collect();

    quote!( #(#field_defs)* )
}

enum FieldAttr {
    Store(syn::Type),
    // Rename(String),
}

impl Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let b = input.peek(syn::Ident);
        println!("----- TYP: {b:?}");

        let ident = syn::Ident::parse(input)?;
        let _eq = proc_macro2::Punct::parse(input)?;
        match ident.to_string().as_str() {
            "store" => {
                let store = syn::Type::parse(input)?;
                Ok(FieldAttr::Store(store))
            }
            _ => Err(Error::new_spanned(
                ident.clone(),
                format!("Invalid field attribute: {ident}"),
            )),
        }
    }
}

impl FieldAttr {
    fn to_tokenstream(&self, field_name: Option<syn::Ident>) -> proc_macro2::TokenStream {
        match self {
            FieldAttr::Store(ty) => quote! { #field_name: #ty, },
        }
    }
}

/*
#[doc(hidden)]
#[allow(non_upper_case_globals,unused_attributes,unused_qualifications)]
const _:() = {
  #[allow(unused_extern_crates,clippy::useless_attribute)]
  extern crate serde as _serde;
  #[allow(unused_macros)]
  macro_rules! try {
    ($__expr:expr) => {
      match$__expr {
        _serde::__private::Ok(__val) => __val,_serde::__private::Err(__err) => {
          return _serde::__private::Err(__err);
        }
      }
    }
  }
  #[automatically_derived]
  impl _serde::Serialize for Device {
    fn serialize<__S>(&self,__serializer:__S) -> _serde::__private::Result<__S::Ok,__S::Error>where __S:_serde::Serializer,{
      let mut __serde_state = try!(_serde::Serializer::serialize_struct(__serializer,"Device",false as usize+1+1));
      try!(_serde::ser::SerializeStruct::serialize_field(&mut __serde_state,"name", &self.name));
      try!(_serde::ser::SerializeStruct::serialize_field(&mut __serde_state,"ip", &self.ip));
      _serde::ser::SerializeStruct::end(__serde_state)
    }

    }

  };
  -----------------------------------------
  #[doc(hidden)]
#[allow(non_upper_case_globals,unused_attributes,unused_qualifications)]
const _:() = {
  #[allow(unused_extern_crates,clippy::useless_attribute)]
  extern crate serde as _serde;
  #[allow(unused_macros)]
  macro_rules! try {
    ($__expr:expr) => {
      match$__expr {
        _serde::__private::Ok(__val) => __val,_serde::__private::Err(__err) => {
          return _serde::__private::Err(__err);
        }
      }
    }
  }
  #[automatically_derived]
  impl <'de>_serde::Deserialize<'de>for Device {
    fn deserialize<__D>(__deserializer:__D) -> _serde::__private::Result<Self,__D::Error>where __D:_serde::Deserializer<'de> ,{
      #[allow(non_camel_case_types)]
      enum __Field {
        __field0,__field1,__ignore,
      }
      struct __FieldVisitor;

      impl <'de>_serde::de::Visitor<'de>for __FieldVisitor {
        type Value = __Field;
        fn expecting(&self,__formatter: &mut _serde::__private::Formatter) -> _serde::__private::fmt::Result {
          _serde::__private::Formatter::write_str(__formatter,"field identifier")
        }
        fn visit_u64<__E>(self,__value:u64) -> _serde::__private::Result<Self::Value,__E>where __E:_serde::de::Error,{
          match __value {
            0u64 => _serde::__private::Ok(__Field::__field0),
            1u64 => _serde::__private::Ok(__Field::__field1),
            _ => _serde::__private::Ok(__Field::__ignore),

            }
        }
        fn visit_str<__E>(self,__value: &str) -> _serde::__private::Result<Self::Value,__E>where __E:_serde::de::Error,{
          match __value {
            "name" => _serde::__private::Ok(__Field::__field0),
            "ip" => _serde::__private::Ok(__Field::__field1),
            _ => {
              _serde::__private::Ok(__Field::__ignore)
            }

            }
        }
        fn visit_bytes<__E>(self,__value: &[u8]) -> _serde::__private::Result<Self::Value,__E>where __E:_serde::de::Error,{
          match __value {
            b"name" => _serde::__private::Ok(__Field::__field0),
            b"ip" => _serde::__private::Ok(__Field::__field1),
            _ => {
              _serde::__private::Ok(__Field::__ignore)
            }

            }
        }

        }
      impl <'de>_serde::Deserialize<'de>for __Field {
        #[inline]
        fn deserialize<__D>(__deserializer:__D) -> _serde::__private::Result<Self,__D::Error>where __D:_serde::Deserializer<'de> ,{
          _serde::Deserializer::deserialize_identifier(__deserializer,__FieldVisitor)
        }

        }
      struct __Visitor<'de>{
        marker:_serde::__private::PhantomData<Device> ,lifetime:_serde::__private::PhantomData< &'de()> ,
      }
      impl <'de>_serde::de::Visitor<'de>for __Visitor<'de>{
        type Value = Device;
        fn expecting(&self,__formatter: &mut _serde::__private::Formatter) -> _serde::__private::fmt::Result {
          _serde::__private::Formatter::write_str(__formatter,"struct Device")
        }
        #[inline]
        fn visit_seq<__A>(self,mut __seq:__A) -> _serde::__private::Result<Self::Value,__A::Error>where __A:_serde::de::SeqAccess<'de> ,{
          let __field0 = match try!(_serde::de::SeqAccess::next_element:: <String>(&mut __seq)){
            _serde::__private::Some(__value) => __value,
            _serde::__private::None => {
              return _serde::__private::Err(_serde::de::Error::invalid_length(0usize, &"struct Device with 2 elements"));
            }

            };
          let __field1 = match try!(_serde::de::SeqAccess::next_element:: <String>(&mut __seq)){
            _serde::__private::Some(__value) => __value,
            _serde::__private::None => {
              return _serde::__private::Err(_serde::de::Error::invalid_length(1usize, &"struct Device with 2 elements"));
            }

            };
          _serde::__private::Ok(Device {
            name:__field0,ip:__field1
          })
        }
        #[inline]
        fn visit_map<__A>(self,mut __map:__A) -> _serde::__private::Result<Self::Value,__A::Error>where __A:_serde::de::MapAccess<'de> ,{
          let mut __field0:_serde::__private::Option<String>  = _serde::__private::None;
          let mut __field1:_serde::__private::Option<String>  = _serde::__private::None;
          while let _serde::__private::Some(__key) = try!(_serde::de::MapAccess::next_key:: <__Field>(&mut __map)){
            match __key {
              __Field::__field0 => {
                if _serde::__private::Option::is_some(&__field0){
                  return _serde::__private::Err(<__A::Error as _serde::de::Error> ::duplicate_field("name"));
                }__field0 = _serde::__private::Some(try!(_serde::de::MapAccess::next_value:: <String>(&mut __map)));
              }
              __Field::__field1 => {
                if _serde::__private::Option::is_some(&__field1){
                  return _serde::__private::Err(<__A::Error as _serde::de::Error> ::duplicate_field("ip"));
                }__field1 = _serde::__private::Some(try!(_serde::de::MapAccess::next_value:: <String>(&mut __map)));
              }
              _ => {
                let _ = try!(_serde::de::MapAccess::next_value:: <_serde::de::IgnoredAny>(&mut __map));
              }

              }
          }let __field0 = match __field0 {
            _serde::__private::Some(__field0) => __field0,
            _serde::__private::None => try!(_serde::__private::de::missing_field("name")),

            };
          let __field1 = match __field1 {
            _serde::__private::Some(__field1) => __field1,
            _serde::__private::None => try!(_serde::__private::de::missing_field("ip")),

            };
          _serde::__private::Ok(Device {
            name:__field0,ip:__field1
          })
        }

        }
      const FIELDS: &'static[&'static str] =  &["name","ip"];
      _serde::Deserializer::deserialize_struct(__deserializer,"Device",FIELDS,__Visitor {
        marker:_serde::__private::PhantomData:: <Device> ,lifetime:_serde::__private::PhantomData,
      })
    }

    }

  };
 */
