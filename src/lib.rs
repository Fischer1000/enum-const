extern crate proc_macro;

use syn::spanned::Spanned;


#[proc_macro_attribute]
pub fn enum_const(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: syn::DeriveInput = match syn::parse(item) {
        Ok(input) => input,
        Err(err) => return syn::Error::new(
            err.span(),
            "only enums with unit variants are supported"
        ).into_compile_error().into()
    };

    let ident = &input.ident;

    input.attrs.retain(|attr| !attr.path().is_ident("repr"));

    match &input.data {
        syn::Data::Enum( syn::DataEnum { variants, .. } ) => {
            let mut match_inner = proc_macro2::TokenStream::new();

            for variant in variants {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    return syn::Error::new(
                        variant.span(),
                        "only unit variants are supported"
                    ).into_compile_error().into();
                }

                let v_ident = &variant.ident;

                match_inner.extend(quote::quote! {
                    x if x == #ident::#v_ident as u8 => Ok(#ident::#v_ident),
                });
            }

            quote::quote!{
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
            }.into()
        },
        syn::Data::Struct( syn::DataStruct { struct_token, .. } ) => {
            syn::Error::new(
                struct_token.span,
                "only enums with unit variants are supported"
            ).into_compile_error().into()
        },
        syn::Data::Union( syn::DataUnion { union_token, .. } ) => {
            syn::Error::new(
                union_token.span,
                "only enums with unit variants are supported"
            ).into_compile_error().into()
        }
    }
}