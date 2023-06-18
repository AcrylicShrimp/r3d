use mlua::prelude::*;

pub trait LuaApiTable {
    fn create_api_table<'lua>(lua: &'lua Lua) -> LuaResult<LuaTable<'lua>>;
}
