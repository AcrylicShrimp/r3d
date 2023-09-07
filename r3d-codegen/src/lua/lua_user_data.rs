use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, LitStr, Meta};

pub fn lua_user_data(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let input = if let Data::Struct(input) = &derive.data {
        input
    } else {
        return TokenStream::new();
    };

    let ty_name = &derive.ident;
    let mut field_getter_impls = Vec::new();
    let mut field_setter_impls = Vec::new();

    for field in &input.fields {
        if field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("hidden"))
            .is_some()
        {
            continue;
        }

        let ident = if let Some(ident) = &field.ident {
            ident
        } else {
            continue;
        };
        let lua_ident = if let Some(attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("rename"))
        {
            match &attr.meta {
                Meta::List(list) => match list.parse_args::<LitStr>() {
                    Ok(lit_str) => lit_str.value(),
                    Err(err) => {
                        abort_call_site!(err);
                    }
                },
                Meta::NameValue(name_value) => match &name_value.value {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Str(lit_str) => lit_str.value(),
                        _ => abort_call_site!(
                            "the argument of rename attribute must be a string literal"
                        ),
                    },
                    _ => abort_call_site!(
                        "the argument of rename attribute must be a string literal"
                    ),
                },
                _ => ident.to_string(),
            }
        } else {
            ident.to_string()
        };

        if field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("use_getter"))
            .is_some()
        {
            field_getter_impls.push(quote! {
                _fields.add_field_method_get(#lua_ident, |lua, this| {
                    <_ as crate::scripting::ConversionByValueReadOnly>::perform_convertion_to_lua(&this.#ident(), lua)
                });
            });
        } else {
            field_getter_impls.push(quote! {
                _fields.add_field_method_get(#lua_ident, |lua, this| {
                    <_ as crate::scripting::ConversionByValueReadOnly>::perform_convertion_to_lua(&this.#ident, lua)
                });
            });
        }

        if field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("readonly"))
            .is_none()
        {
            if field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("use_setter"))
                .is_some()
            {
                let setter_ident = format_ident!("set_{}", ident);
                field_setter_impls.push(quote! {
                    _fields.add_field_method_set(#lua_ident, |lua, this, value| {
                        this.#setter_ident(<_ as crate::scripting::ConversionByValue>::perform_conversion_from_lua(value, lua)?);
                        Ok(())
                    });
                });
            } else {
                field_setter_impls.push(quote! {
                    _fields.add_field_method_set(#lua_ident, |lua, this, value| {
                        this.#ident = <_ as crate::scripting::ConversionByValue>::perform_conversion_from_lua(value, lua)?;
                        Ok(())
                    });
                });
            }
        }
    }

    let into_self_impl = if derive
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("impl_copy"))
        .is_some()
    {
        quote! {
            Ok(*user_data.borrow::<Self>()?)
        }
    } else {
        quote! {
            user_data.take()
        }
    };

    let generics = &derive.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics mlua::UserData for #ty_name #ty_generics #where_clause {
            fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {
                #(#field_getter_impls)*
                #(#field_setter_impls)*
            }

            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(_methods: &mut M) {
                <Self as crate::scripting::UserDataMethodProvider>::add_methods(_methods);
            }
        }

        impl #impl_generics crate::scripting::LuaType for #ty_name #ty_generics #where_clause {
            type LuaType = Self;
        }

        impl #impl_generics crate::scripting::LuaTypeToOriginal for <#ty_name #ty_generics as crate::scripting::LuaType>::LuaType #where_clause {
            type OriginalType = #ty_name #ty_generics;

            fn from_original(original: Self::OriginalType) -> Self {
                original
            }

            fn as_original(&self) -> &Self::OriginalType {
                self
            }
        }

        impl #impl_generics crate::scripting::LuaTypeToOriginalMut for <#ty_name #ty_generics as crate::scripting::LuaType>::LuaType #where_clause {
            fn as_original_mut(&mut self) -> &mut Self::OriginalType {
                self
            }
        }

        impl #impl_generics crate::scripting::ConversionByValueReadOnly for <#ty_name #ty_generics as crate::scripting::LuaType>::LuaType #where_clause {
            fn perform_convertion_to_lua<'lua>(&self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
               <_ as mlua::IntoLua>::into_lua(self.clone(), lua)
            }
        }

        impl #impl_generics crate::scripting::ConversionByValue for <#ty_name #ty_generics as crate::scripting::LuaType>::LuaType #where_clause {
            fn perform_conversion_from_lua<'lua>(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(user_data) => Ok(user_data.take()?),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: "userdata",
                        to: stringify!(#ty_name),
                        message: Some(format!("expected a userdata, got [{}]", value.type_name())),
                    }),
                }
            }
        }

        impl #impl_generics crate::scripting::UserDataIntoSelf for <#ty_name #ty_generics as crate::scripting::LuaType>::LuaType #where_clause {
            fn into_self<'lua>(user_data: mlua::AnyUserData<'lua>) -> mlua::Result<Self> {
                #into_self_impl
            }
        }
    })
}
