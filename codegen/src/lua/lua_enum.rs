use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

pub fn lua_enum(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let input = if let Data::Enum(input) = &derive.data {
        input
    } else {
        return TokenStream::new();
    };

    let ty_name = &derive.ident;
    let mut to_string_impls = Vec::new();
    let mut api_table_impls = Vec::new();

    for variant in &input.variants {
        let ident = &variant.ident;
        let variant_in_str = format!("{}::{}", ty_name, ident);
        to_string_impls.push(quote! {
            #ty_name::#ident => Ok(#variant_in_str),
        });
        api_table_impls.push(quote! {
            table.set(stringify!(#ident), #ty_name::#ident)?;
        });
    }

    let generics = &derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics mlua::UserData for #ty_name #ty_generics #where_clause {
            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
                methods.add_meta_method(mlua::MetaMethod::ToString, |_lua, this, ()| match this {
                    #(#to_string_impls)*
                });
                methods.add_meta_function(mlua::MetaMethod::Eq, |_lua, (lhs, rhs): (Self, Self)| {
                    Ok(lhs == rhs)
                });
            }
        }

        impl #impl_generics crate::engine::scripting::LuaType for #ty_name #ty_generics #where_clause {
            type LuaType = Self;
        }

        impl #impl_generics crate::engine::scripting::LuaTypeToOriginal for <#ty_name #ty_generics as crate::engine::scripting::LuaType>::LuaType #where_clause {
            type OriginalType = #ty_name #ty_generics;

            fn from_original(original: Self::OriginalType) -> Self {
                original
            }

            fn as_original(&self) -> &Self::OriginalType {
                self
            }
        }

        impl #impl_generics crate::engine::scripting::LuaTypeToOriginalMut for <#ty_name #ty_generics as crate::engine::scripting::LuaType>::LuaType #where_clause {
            fn as_original_mut(&mut self) -> &mut Self::OriginalType {
                self
            }
        }

        impl #impl_generics crate::engine::scripting::ConversionByValueReadOnly for <#ty_name #ty_generics as crate::engine::scripting::LuaType>::LuaType #where_clause {
            fn perform_convertion_to_lua<'lua>(&self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
                <Self as mlua::ToLua>::to_lua(self.clone(), lua)
            }
        }

        impl #impl_generics crate::engine::scripting::ConversionByValue for <#ty_name #ty_generics as crate::engine::scripting::LuaType>::LuaType #where_clause {
            fn perform_conversion_from_lua<'lua>(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
                <Self as mlua::FromLua>::from_lua(value, lua)
            }
        }

        impl #impl_generics crate::engine::scripting::LuaApiTable for <#ty_name #ty_generics as crate::engine::scripting::LuaType>::LuaType #where_clause {
            fn create_api_table<'lua>(lua: &'lua mlua::Lua) -> mlua::Result<mlua::Table<'lua>> {
                let table = lua.create_table()?;
                #(#api_table_impls)*
                Ok(table)
            }
        }
    })
}
