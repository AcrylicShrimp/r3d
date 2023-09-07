use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

pub fn handle(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let ty_name = &derive.ident;
    let handle_name = format_ident!("{}Handle", ty_name);
    let generics = &derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        #[derive(Clone)]
        pub struct #handle_name #generics #where_clause {
            inner: std::sync::Arc<#ty_name #ty_generics>,
        }

        impl #impl_generics #handle_name #ty_generics #where_clause {
            pub fn new(inner: #ty_name #ty_generics) -> Self {
                Self {
                    inner: std::sync::Arc::new(inner),
                }
            }
        }

        impl #impl_generics std::ops::Deref for #handle_name #ty_generics #where_clause {
            type Target = #ty_name #ty_generics;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl #impl_generics PartialEq for #handle_name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                std::sync::Arc::ptr_eq(&self.inner, &other.inner)
            }
        }

        impl #impl_generics Eq for #handle_name #ty_generics #where_clause {}

        impl #impl_generics std::hash::Hash for #handle_name #ty_generics #where_clause {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                std::sync::Arc::as_ptr(&self.inner).hash(state);
            }
        }
    })
}
