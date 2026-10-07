extern crate proc_macro;

use syn::spanned::Spanned;


#[proc_macro_attribute]
pub fn enum_const(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_const_impl(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}


fn enum_const_impl(_attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: syn::DeriveInput = match syn::parse2(item) {
        Ok(input) => input,
        Err(err) => {
            println!("{err}");
            return Err(syn::Error::new(
                err.span(),
                "expected an enum",
            ))
        }
    };

    for attr in input.attrs.iter() {
        if attr.path().is_ident("repr") {
            return Err(syn::Error::new(
                attr.span(),
                "this macro implicitly adds a `#[repr(u8)]` attribute",
            ))
        }
    }

    let ident = &input.ident;

    match &input.data {
        syn::Data::Enum( syn::DataEnum { variants, .. } ) => {
            let mut match_inner = proc_macro2::TokenStream::new();

            for variant in variants {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    return Err(syn::Error::new(
                        variant.span(),
                        "expected a unit variant"
                    ))
                }

                let v_ident = &variant.ident;

                match_inner.extend(quote::quote! {
                    x if x == #ident::#v_ident as u8 => Ok(#ident::#v_ident),
                });
            }

            Ok(quote::quote!{
                #[repr(u8)]
                #input

                #[automatically_derived]
                impl From<#ident> for u8 {
                    fn from(value: #ident) -> Self {
                        value as u8
                    }
                }

                #[automatically_derived]
                impl TryFrom<u8> for #ident {
                    type Error = ();

                    fn try_from(value: u8) -> Result<Self, Self::Error> {
                        match value {
                            #match_inner
                            _ => Err(()),
                        }
                    }
                }
            }.into())
        },
        syn::Data::Struct( syn::DataStruct { struct_token, .. } ) => {
            Err(syn::Error::new(
                struct_token.span,
                "expected an enum"
            ))
        },
        syn::Data::Union( syn::DataUnion { union_token, .. } ) => {
            Err(syn::Error::new(
                union_token.span,
                "expected an enum"
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_ok(input: proc_macro2::TokenStream) {
        let result = enum_const_impl(proc_macro2::TokenStream::new(), input);

        assert!(result.is_ok());

        println!("{}", result.unwrap_or_else(|e| e.to_compile_error()));
    }

    fn test_err(input: proc_macro2::TokenStream) {
        let result = enum_const_impl(proc_macro2::TokenStream::new(), input);

        assert!(result.is_err());

        println!("{}", result.unwrap_or_else(|e| e.to_compile_error()));
    }

    #[test]
    fn unit_struct() {
        let input = quote::quote!{
            struct Unit;
        };

        test_err(input);
    }

    #[test]
    fn tuple_struct() {
        let input = quote::quote!{
            struct Tuple(i32, i32);
        };

        test_err(input);
    }

    #[test]
    fn named_struct() {
        let input = quote::quote!{
            struct Named {
                x: f32,
                y: f32
            }
        };

        test_err(input);
    }

    #[test]
    fn mixed_enum() {
        let input = quote::quote!{
            enum Mixed {
                A,
                B(i32, i32),
                C { x: f32, y: f32 },
                D(),
                E { }
            }
        };

        test_err(input);
    }

    #[test]
    fn unit_enum() {
        let input = quote::quote!{
            enum Unit {
                A,
                B,
                C,
                D
            }
        };

        test_ok(input);
    }

    #[test]
    fn with_repr() {
        let input = quote::quote!{
            #[repr(i16)]
            enum WithRepr {
                A,
                B,
                C,
                D
            }
        };

        test_err(input);
    }
}