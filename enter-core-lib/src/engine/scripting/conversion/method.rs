use mlua::prelude::*;

pub trait UserDataMethodProvider
where
    Self: LuaUserData,
{
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

pub trait UserDataOpsProvider
where
    Self: LuaUserData,
{
    fn add_ops<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
}
