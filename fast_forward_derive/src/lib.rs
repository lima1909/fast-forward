use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Indexed, attributes(index))]
pub fn indexed(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    println!("AST:\n {ast:#?}");

    if let Some(index) = ast.attrs.first() {
        if let syn::Meta::List(ref l) = index.meta {
            let clone = l.tokens.clone();
            // println!("ATTR:\n {:#?}", l.tokens);
            return quote::quote!( let _c: #clone; ).into();
        }
    }

    quote::quote!(
            mod __bar {

            use super::*;

            pub struct Bar {
                a: i32,
            }

            impl Bar {
                pub fn new(a: i32) -> Self {
                    Self { a }
                }

                pub fn foo(&self, _f: First) {}
            }

        }

        pub use __bar::Bar;

        impl First {
            fn foo(&self)-> &str {
                &self.name
            }
        }
    )
    .into()
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
