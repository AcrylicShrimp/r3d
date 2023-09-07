use mlua::prelude::*;

pub trait UserDataIntoSelf
where
    Self: LuaUserData,
{
    fn into_self<'lua>(user_data: LuaAnyUserData<'lua>) -> LuaResult<Self>;
}
