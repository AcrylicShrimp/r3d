use mlua::prelude::*;

pub trait LuaType: Sized {
    type LuaType: LuaUserData + Clone;
}

pub trait LuaTypeToOriginal {
    type OriginalType: LuaType<LuaType = Self>;

    fn from_original(original: Self::OriginalType) -> Self;
    fn as_original(&self) -> &Self::OriginalType;
}

pub trait LuaTypeToOriginalMut: LuaTypeToOriginal {
    fn as_original_mut(&mut self) -> &mut Self::OriginalType;
}

pub trait LuaTypeToOriginalInnerMut: LuaTypeToOriginal {
    fn as_original_mut(&self) -> &mut Self::OriginalType;
}
