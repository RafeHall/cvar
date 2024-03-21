#![feature(proc_macro_diagnostic)]

use convert_case::Casing;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, ItemEnum};

#[derive(Debug)]
struct CVarEnumAttribute {
    pub name: syn::Ident,
    pub _assign: syn::Token![=],
    pub value: syn::Lit,
}

impl Parse for CVarEnumAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let assign: syn::Token![=] = input.parse()?;
        let value = input.parse()?;

        Ok(CVarEnumAttribute {
            name,
            _assign: assign,
            value,
        })
    }
}

#[proc_macro_derive(CVarEnum, attributes(cvar))]
pub fn c_var_enum(tokens: TokenStream) -> TokenStream {
    let ItemEnum {
        ident, variants, ..
    } = syn::parse_macro_input!(tokens as ItemEnum);

    variants.iter().for_each(|variant| match variant.fields {
        syn::Fields::Unit => {}
        ref f => f
            .span()
            .unwrap()
            .error("CVarEnum can only be implemented on enum with unit types only")
            .emit(),
    });

    let idents: Vec<_> = variants
        .iter()
        .flat_map(|variant| {
            let mut variants = vec![(variant.ident.clone(), variant.ident.to_string())];

            for attr in &variant.attrs {
                let attr = attr.parse_args_with(CVarEnumAttribute::parse).unwrap();

                if attr.name == "alias" {
                    if let syn::Lit::Str(value) = attr.value {
                        variants.push((variant.ident.clone(), value.value()));
                    } else {
                        attr.value
                            .span()
                            .unwrap()
                            .error("expected string literal for alias")
                            .emit()
                    }
                }
            }

            variants
        })
        .map(|(ident, name)| (ident, name.to_case(convert_case::Case::Snake)))
        .collect();

    let (idents, names): (Vec<_>, Vec<String>) = idents.into_iter().unzip();
    let count = idents.len();

    quote! {
        impl cvars::Value for #ident {
            fn parse(s: &str) -> Result<Self, cvars::Error> {
                let r = match s {
                    #(
                        #names => #ident::#idents,
                    )*
                    "" => return Err(cvars::Error::EmptyValue),
                    _ => return Err(cvars::Error::invalid_value(s)),
                };

                Ok(r)
            }

            fn validate(s: &str) -> Result<Vec<String>, cvars::Error> {
                const VALUES: [&str; #count] = [
                    #(
                        #names
                    ),*
                ];

                let mut values = vec![];

                for value in VALUES {
                    if value.starts_with(s) {
                        values.push(value.to_string());
                    }
                }

                Ok(values)
            }
        }
    }
    .into()
}
