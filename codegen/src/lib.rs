mod lua;

use proc_macro::TokenStream;
use proc_macro_error::*;

#[proc_macro_derive(LuaEnum)]
#[proc_macro_error]
pub fn lua_enum(item: TokenStream) -> TokenStream {
    lua::lua_enum(item)
}

#[proc_macro_derive(LuaError)]
#[proc_macro_error]
pub fn lua_error(item: TokenStream) -> TokenStream {
    lua::lua_error(item)
}

#[proc_macro_derive(
    LuaUserData,
    attributes(impl_copy, readonly, hidden, rename, use_getter, use_setter)
)]
#[proc_macro_error]
pub fn lua_user_data(item: TokenStream) -> TokenStream {
    lua::lua_user_data(item)
}

#[proc_macro_derive(
    LuaUserDataArc,
    attributes(impl_copy, readonly, hidden, rename, use_getter, use_setter)
)]
#[proc_macro_error]
pub fn lua_user_data_arc(item: TokenStream) -> TokenStream {
    lua::lua_user_data_arc(item)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn lua_user_data_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    lua::lua_user_data_method(attr, item)
}

#[proc_macro_attribute]
pub fn hidden(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn rename(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn ops_to_string(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn ops_extra(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
