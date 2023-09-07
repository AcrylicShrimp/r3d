use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn lua_error(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let ty_name = &derive.ident;

    TokenStream::from(quote! {
        impl From<#ty_name> for mlua::Error {
            fn from(err: #ty_name) -> Self {
                mlua::Error::external(err.to_string())
            }
        }
    })
}
