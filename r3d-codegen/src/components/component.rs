use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn component(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let ty_name = &derive.ident;
    let generics = &derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics crate::object::new::ComponentCommon for #ty_name #ty_generics #where_clause {
            fn type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    })
}
